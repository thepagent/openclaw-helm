# Gatekeeper Sidecar Design

## Overview

Gatekeeper is a sidecar container that runs alongside the OpenClaw main container within the same Kubernetes pod. It acts as the sole holder of secrets, enforcing human-in-the-loop approval via Telegram before returning any secret to OpenClaw.

OpenClaw itself holds **zero secrets** — no env vars, no files, no keys.

## Architecture

```
┌─────────────────── K3s / K8s Cluster ─────────────────────────┐
│                                                                │
│  ┌── AWS Secrets Manager ──┐                                  │
│  │  openclaw/tokens         │                                  │
│  │    TELEGRAM_TOKEN_1      │                                  │
│  │    TELEGRAM_TOKEN_2      │                                  │
│  │    GATEWAY_TOKEN         │                                  │
│  └────────────┬─────────────┘                                  │
│               │ IAM Role (sidecar SA only)                     │
│               ▼                                                │
│  ┌──────────────────── OpenClaw Pod ────────────────────────┐  │
│  │                                                          │  │
│  │  ┌─────────────────────┐    ┌──────────────────────┐    │  │
│  │  │  main (OpenClaw)     │    │  gatekeeper (sidecar) │    │  │
│  │  │                     │    │                      │    │  │
│  │  │  ❌ no secrets       │    │  ✅ IAM Role         │    │  │
│  │  │  ❌ no env keys      │    │  ✅ fetches from AWS  │    │  │
│  │  │  ❌ no AWS access    │    │  ✅ Telegram approval │    │  │
│  │  │                     │    │  ✅ rate limiting     │    │  │
│  │  │  exec: curl unix ───────► /tmp/gatekeeper.sock  │    │  │
│  │  │  socket → get secret│    │         │            │    │  │
│  │  │                     │    │         ▼            │    │  │
│  │  │                     │    │  📱 Telegram notify  │    │  │
│  │  │                     │    │  [✅ Approve][❌ Deny]│    │  │
│  │  │                     │    │         │            │    │  │
│  │  │  ◄── secret returned────────────── ▼            │    │  │
│  │  │  (stored in memory) │    │  fetch from AWS SM   │    │  │
│  │  └─────────────────────┘    └──────────────────────┘    │  │
│  │          shared emptyDir volume (/tmp/gatekeeper.sock)   │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
└────────────────────────────────────────────────────────────────┘

         📱 Your Phone
         ┌──────────────────────┐
         │ 🔐 Gatekeeper Alert  │
         │                      │
         │ OpenClaw requests    │
         │ secret access        │
         │ Time: 09:58          │
         │                      │
         │ [✅ Approve][❌ Deny] │
         └──────────────────────┘
```

## Security Model

| Threat | Mitigation |
|--------|-----------|
| Agent reads env vars | No secrets in main container env |
| Agent reads sidecar filesystem | Different container — filesystem isolated |
| Agent calls unix socket directly | Sidecar sends Telegram alert — you deny |
| Agent runs `aws secretsmanager get-secret-value` | Main container SA has no IAM permissions |
| `kubectl exec` into sidecar | Requires cluster RBAC — agent doesn't have it |
| Brute-force socket requests | Rate limit: max 1 request per 5 minutes; alerts on repeated attempts |

## Components

### 1. AWS Secrets Manager
- Stores all tokens and keys under a single path (e.g. `openclaw/tokens`)
- Supports automatic rotation
- Every access is logged in CloudTrail

### 2. Gatekeeper Sidecar
- Minimal container (Alpine-based)
- Listens on `/tmp/gatekeeper.sock` (shared `emptyDir` volume)
- On request:
  1. Sends Telegram approval message with Approve / Deny buttons
  2. Waits up to `approvalTimeoutSeconds` for response
  3. On approval: fetches secret from AWS Secrets Manager via IAM Role, returns to caller
  4. On denial or timeout: returns error
- Rate limit: configurable, default 1 request per 5 minutes
- Alerts on anomalous repeated requests

### 3. IAM / RBAC Separation
- Gatekeeper uses a dedicated `ServiceAccount` annotated with an IAM Role (IRSA or EKS Pod Identity)
- Main container uses a separate `ServiceAccount` with **no AWS permissions**
- RBAC: neither SA has `exec` rights into pods

## Helm Values

```yaml
gatekeeper:
  enabled: false                  # opt-in
  image:
    repository: ghcr.io/thepagent/openclaw-gatekeeper
    tag: latest
  aws:
    region: ap-northeast-1
    secretsManagerPath: openclaw/tokens
  telegram:
    approvalTimeoutSeconds: 60
    rateLimitMinutes: 5
  serviceAccount:
    annotations:
      eks.amazonaws.com/role-arn: arn:aws:iam::ACCOUNT_ID:role/openclaw-gatekeeper
```

## Repository Layout

```
openclaw-helm/
├── gatekeeper/               # sidecar source code
│   ├── Dockerfile
│   ├── main.py (or main.go)
│   └── requirements.txt
├── templates/
│   ├── deployment.yaml       # injects sidecar when gatekeeper.enabled=true
│   └── serviceaccount.yaml   # separate SA for gatekeeper
├── docs/
│   └── gatekeeper.md         # this document
└── .github/workflows/
    └── gatekeeper-image.yml  # CI: build & push gatekeeper image on change
```

## CI / CD

A GitHub Actions workflow (`gatekeeper-image.yml`) will:
1. Trigger on changes to `gatekeeper/**`
2. Build the Docker image
3. Push to `ghcr.io/thepagent/openclaw-gatekeeper` with the commit SHA tag and `latest`

## Deployment Flow

1. Create IAM Role with `secretsmanager:GetSecretValue` on `openclaw/tokens`
2. Store secrets in AWS Secrets Manager
3. Set `gatekeeper.enabled: true` and configure `values.yaml`
4. `helm upgrade --install openclaw oci://ghcr.io/thepagent/openclaw-helm -f values.yaml`
5. On first OpenClaw startup, approve the Telegram request on your phone

## Future Considerations

- **Audit log**: persist approval/denial events to a local file or CloudWatch
- **Multi-secret support**: allow OpenClaw to request individual named secrets rather than the full bundle
- **mTLS over socket**: replace plain unix socket with mTLS for stronger channel integrity
