# Backup

> **Note:** This document is intended to be read together with your AI agent. Share it directly and ask your agent to implement the scripts for your specific environment.

All OpenClaw state lives in `/home/node/.openclaw` inside the pod, including:
- `openclaw.json` — main configuration
- Channel credentials and pairing tokens
- Memory and session data
- Installed skills

## Concept

The backup strategy has two layers:

**Layer 1 — Local backup**
A scheduled OS cron job runs on the host machine. It uses `openclaw backup create` inside the pod — introduced in **OpenClaw 2026.3.8** — to produce a verified archive with an embedded manifest, then copies it to the host via `kubectl cp`. Old backups beyond your retention window are pruned automatically.

**Layer 2 — Remote sync**
After the local backup completes, the backup directory is synced to a remote destination (cloud storage, another host, etc.) for off-site durability.

A ready-made script is provided at `scripts/openclaw-backup.sh`.

```
╔═════════════════════════════════════════════════════╗
║              cron (daily)                           ║
║     scripts/openclaw-backup.sh                      ║
╚════════════════════╤════════════════════════════════╝
                     │
                     ▼
         ╔═══════════════════════╗
         ║  openclaw backup      ║
         ║  create --verify      ║
         ║  (inside pod)         ║
         ╚═══════════╤═══════════╝
                     │
                     ▼
         ╔═══════════════════════╗
         ║  kubectl cp           ║
         ║  archive → host       ║
         ╚═══════════╤═══════════╝
                     │
                     ▼
         ╔═══════════════════════╗
         ║  prune old backups    ║
         ║  (retention window)   ║
         ╚═══════════╤═══════════╝
                     │
                     ▼
         ╔═══════════════════════╗
         ║  sync to remote       ║
         ║  (S3 / R2 / rsync /   ║
         ║   rclone / etc.)      ║
         ╚═══════════════════════╝
```

## Quick Start (k3s)

```bash
# Run once to verify
bash scripts/openclaw-backup.sh

# Schedule daily at 2am
(crontab -l; echo "0 2 * * * bash $HOME/repo/openclaw-helm/scripts/openclaw-backup.sh >> $HOME/Backups/openclaw/backup.log 2>&1") | crontab -

# Optional: enable S3 sync
export REMOTE_DEST=s3://your-bucket/openclaw-backups
```

## Restore

To restore from a backup:

1. Copy the backup directory back into the pod with `kubectl cp`
2. Restart the deployment with `kubectl rollout restart`

## Notes

- The Helm chart will **not** override an existing `openclaw.json` on the PVC, but always back up before upgrading.
- For k3s, prefix `kubectl` commands with `sudo -E` and set `KUBECONFIG=/etc/rancher/k3s/k3s.yaml`.
