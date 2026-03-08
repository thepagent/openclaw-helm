# Backup Strategy

This document describes the recommended backup strategy for OpenClaw deployed via Helm.

## What to Back Up

All OpenClaw state lives in `/home/node/.openclaw` inside the pod, including:
- `openclaw.json` — main configuration
- Channel credentials and pairing tokens
- Memory and session data
- Installed skills

## Concept

The backup strategy has two layers:

**Layer 1 — Local backup**
A scheduled OS cron job runs on the host machine, pulls `/home/node/.openclaw` from the pod via `kubectl cp`, and stores it locally with a timestamp. Old backups beyond your retention window are pruned automatically.

**Layer 2 — Remote sync**
After the local backup completes, the backup directory is synced to a remote destination (cloud storage, another host, etc.) for off-site durability.

We do not provide ready-made scripts here. Share this guide with your AI agent and ask it to write scripts tailored to your environment and tooling (e.g. `aws s3 sync`, `rclone`, `rsync` over SSH).

## Auto-Upgrade with Pre-Upgrade Backup

A common pattern is to combine backup with automated Helm upgrades. An auto-upgrade checker runs on a cron schedule, compares the latest OCI registry version against the currently deployed version, and only proceeds when a new version is detected — backing up first, then upgrading, and sending a notification on success:

```text
# Workflow:
#
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

Share this diagram with your AI agent and ask it to implement the full script for your environment.

## Restore

To restore from a backup:

1. Copy the backup directory back into the pod with `kubectl cp`
2. Restart the deployment with `kubectl rollout restart`

## Notes

- The Helm chart will **not** override an existing `openclaw.json` on the PVC, but always back up before upgrading.
- For k3s, prefix `kubectl` commands with `sudo -E` and set `KUBECONFIG=/etc/rancher/k3s/k3s.yaml`.
