# Persistent Tools in Kubernetes Deployments

When OpenClaw runs as a Helm chart in a Kubernetes environment, it runs inside a Pod where **only `~/.openclaw` is persisted** to a PVC. Everything else is ephemeral and lost on Pod restart or upgrade.

## The Golden Rule

> Install all CLI tools and binaries under `~/.openclaw/workspace/bin/`, not system paths.

`~/.openclaw/workspace` is the default agent workspace directory. It is **not** exposed as a `$WORKSPACE` environment variable — use the full path explicitly.

## Why

- The Pod has **no `sudo`** and no `apt-get`
- System paths (`/usr/local/bin`, `/usr/bin`, etc.) are reset on every restart
- Only `~/.openclaw/**` survives restarts and upgrades via PVC

## Installing Tools

Instead of `apt install`, ask OpenClaw to **download the official binary directly into `~/.openclaw/workspace/bin/`**.

Examples:

```
~/.openclaw/workspace/bin/gh        # GitHub CLI
~/.openclaw/workspace/bin/wrangler  # Cloudflare CLI
~/.openclaw/workspace/bin/weather   # Weather CLI
```

### Example prompt to OpenClaw

> Download the latest GitHub CLI Linux amd64 binary and install it to `~/.openclaw/workspace/bin/gh`

OpenClaw will fetch the release tarball, extract the binary, and place it at the correct path.

## Using Installed Tools

Ensure `~/.openclaw/workspace/bin` is on `PATH` before running:

```bash
export PATH=~/.openclaw/workspace/bin:$PATH
gh --version
```

You can also ask OpenClaw to prepend this in any command it runs, or add it to a skill's instructions.

## npm / Node Packages

For npm-based CLIs, install to the workspace as well:

```bash
npm install -g --prefix ~/.openclaw/workspace <package>
# binary lands at ~/.openclaw/workspace/bin/<package>
```

## Authorizing CLIs (Login / OAuth)

The Pod has no browser, so any CLI that requires login must use **device flow**.

Simply tell OpenClaw:

> "Log in to `<CLI name>` using device flow and send me the link with the code."

OpenClaw will run the command, capture the one-time code and URL, and send them to you via Telegram or Discord so you can authorize from your phone.

## Summary

| ✅ Persisted | ❌ Not Persisted |
|---|---|
| `~/.openclaw/**` | `/usr/local/bin/` |
| `~/.openclaw/workspace/bin/` | `/usr/bin/` |
| `~/.openclaw/workspace/node_modules/` | System packages (`apt`) |

As long as binaries live under `~/.openclaw`, they survive Pod restarts and Helm upgrades.
