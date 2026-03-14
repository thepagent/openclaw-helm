use anyhow::Result;
use aws_sdk_secretsmanager::Client as SmClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tracing::{info, warn};
use zeroize::Zeroizing;

const SOCKET_PATH: &str = "/tmp/gatekeeper.sock";
const RATE_LIMIT_SECS: u64 = 300; // 5 minutes

#[derive(Deserialize)]
struct SecretRequest {
    name: String,
}

#[derive(Serialize)]
struct SecretResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

struct State {
    sm_client: SmClient,
    tg_bot_token: String,
    tg_chat_id: String,
    last_request: Option<Instant>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let tg_bot_token = std::env::var("GATEKEEPER_TG_BOT_TOKEN")
        .expect("GATEKEEPER_TG_BOT_TOKEN required");
    let tg_chat_id = std::env::var("GATEKEEPER_TG_CHAT_ID")
        .expect("GATEKEEPER_TG_CHAT_ID required");

    let aws_cfg = aws_config::load_from_env().await;
    let sm_client = SmClient::new(&aws_cfg);

    let state = Arc::new(Mutex::new(State {
        sm_client,
        tg_bot_token,
        tg_chat_id,
        last_request: None,
    }));

    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("gatekeeper listening on {}", SOCKET_PATH);

    loop {
        let (stream, _) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle(stream, state).await {
                warn!("handle error: {e}");
            }
        });
    }
}

async fn handle(stream: tokio::net::UnixStream, state: Arc<Mutex<State>>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut line = String::new();
    BufReader::new(reader).read_line(&mut line).await?;

    let req: SecretRequest = serde_json::from_str(line.trim())?;

    let resp = {
        let mut s = state.lock().await;

        // rate limit
        if let Some(last) = s.last_request {
            if last.elapsed() < Duration::from_secs(RATE_LIMIT_SECS) {
                let remaining = RATE_LIMIT_SECS - last.elapsed().as_secs();
                warn!("rate limited — {} seconds remaining", remaining);
                SecretResponse {
                    value: None,
                    error: Some(format!("rate limited, retry in {remaining}s")),
                }
            } else {
                fetch_with_approval(&mut s, &req.name).await
            }
        } else {
            fetch_with_approval(&mut s, &req.name).await
        }
    };

    let mut out = serde_json::to_string(&resp)?;
    out.push('\n');
    writer.write_all(out.as_bytes()).await?;
    Ok(())
}

async fn fetch_with_approval(s: &mut State, secret_name: &str) -> SecretResponse {
    s.last_request = Some(Instant::now());

    // send Telegram approval request
    let approved = match request_approval(&s.tg_bot_token, &s.tg_chat_id, secret_name).await {
        Ok(v) => v,
        Err(e) => {
            warn!("telegram error: {e}");
            return SecretResponse { value: None, error: Some("telegram error".into()) };
        }
    };

    if !approved {
        warn!("request denied by operator");
        return SecretResponse { value: None, error: Some("denied".into()) };
    }

    // fetch from AWS Secrets Manager
    match s.sm_client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await
    {
        Ok(out) => {
            let secret = Zeroizing::new(
                out.secret_string().unwrap_or_default().to_string()
            );
            info!("secret '{}' returned to caller", secret_name);
            SecretResponse { value: Some(secret.to_string()), error: None }
        }
        Err(e) => {
            warn!("secrets manager error: {e}");
            SecretResponse { value: None, error: Some("aws error".into()) }
        }
    }
}

/// Sends a Telegram message with Approve/Deny inline buttons and polls for the answer.
/// Returns true if approved within 60 seconds.
async fn request_approval(bot_token: &str, chat_id: &str, secret_name: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    let base = format!("https://api.telegram.org/bot{bot_token}");

    // send message
    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": format!("🔐 Gatekeeper: secret access requested\n\nSecret: `{secret_name}`\n\nApprove?"),
        "parse_mode": "Markdown",
        "reply_markup": {
            "inline_keyboard": [[
                {"text": "✅ Approve", "callback_data": "approve"},
                {"text": "❌ Deny",    "callback_data": "deny"}
            ]]
        }
    });

    let send_resp: serde_json::Value = client
        .post(format!("{base}/sendMessage"))
        .json(&body)
        .send().await?
        .json().await?;

    let message_id = send_resp["result"]["message_id"].as_i64().unwrap_or(0);

    // poll for callback (max 60s)
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut offset: i64 = 0;

    while Instant::now() < deadline {
        let updates: serde_json::Value = client
            .get(format!("{base}/getUpdates"))
            .query(&[("timeout", "5"), ("allowed_updates", "callback_query")])
            .query(&[("offset", offset.to_string())])
            .send().await?
            .json().await?;

        if let Some(arr) = updates["result"].as_array() {
            for update in arr {
                offset = update["update_id"].as_i64().unwrap_or(0) + 1;
                let cb = &update["callback_query"];
                if cb["message"]["message_id"].as_i64() == Some(message_id) {
                    let data = cb["data"].as_str().unwrap_or("");
                    let cb_id = cb["id"].as_str().unwrap_or("");
                    // acknowledge
                    let _ = client.post(format!("{base}/answerCallbackQuery"))
                        .json(&serde_json::json!({"callback_query_id": cb_id}))
                        .send().await;
                    return Ok(data == "approve");
                }
            }
        }
    }

    Ok(false) // timeout = deny
}
