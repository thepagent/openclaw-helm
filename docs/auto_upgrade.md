# Auto-Upgrade

> **Note:** This document is intended to be read together with your AI agent. Share it directly and ask your agent to implement the scripts for your specific environment.

An auto-upgrade checker runs on a cron schedule, compares the latest OCI registry version against the currently deployed version, and only proceeds when a new version is detected — backing up first, then upgrading, and sending a notification on success.

A sample script is provided at [`scripts/openclaw-autoupgrade.sh`](../scripts/openclaw-autoupgrade.sh). Copy it, set your `--target <YOUR_TELEGRAM_CHAT_ID>`, and schedule it with cron.

## Workflow

```text
# ╔═════════════════════════════════════════════════════╗
# ║              cron (every hour)                      ║
# ║     /path/to/openclaw-autoupgrade.sh                ║
# ╚════════════════════╤════════════════════════════════╝
#                      │
#                      ▼
#          ╔═══════════════════════╗
#          ║  helm pull --dry-run  ║
#          ║  (check OCI registry) ║
#          ╚═══════════╤═══════════╝
#                      │
#                      ▼
#          ╔═══════════════════════╗
#          ║  LATEST vs CURRENT    ║
#          ╚═══════════╤═══════════╝
#                      │
#               ╔══════╧══════╗
#               ║             ║
#          LATEST ==       LATEST !=
#          CURRENT          CURRENT
#               ║             ║
#               ▼             ▼
#          ╔════════╗   ╔═══════════════╗
#          ║log "up ║   ║ backup first  ║
#          ║to date"║   ╚═══════╤═══════╝
#          ╚════════╝           │
#                               ▼
#                    ╔══════════════════════╗
#                    ║ helm upgrade (OCI)   ║
#                    ╚══════════╤═══════════╝
#                               │
#                               ▼
#                    ╔══════════════════════╗
#                    ║ kubectl rollout      ║
#                    ║ status --timeout=120s║
#                    ╚══════════╤═══════════╝
#                               │
#                               ▼
#                    ╔══════════════════════╗
#                    ║ openclaw message send║
#                    ╚══════════╤═══════════╝
#                               │
#                               ▼
#                    ╔══════════════════════╗
#                    ║  Telegram            ║
#                    ║  OpenClaw upgraded   ║
#                    ║  1.3.2 → 1.3.3      ║
#                    ╚══════════════════════╝
```

## Notes

- The backup step (see [backup.md](backup.md)) must complete successfully before the upgrade proceeds.
- For k3s, prefix `kubectl` and `helm` commands with `sudo -E` and set `KUBECONFIG=/etc/rancher/k3s/k3s.yaml`.
