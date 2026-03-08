# Backup Strategy

This document describes the recommended backup strategy for OpenClaw deployed via Helm.

## What to Back Up

All OpenClaw state lives in `/home/node/.openclaw` inside the pod, including:
- `openclaw.json` — main configuration
- Channel credentials and pairing tokens
- Memory and session data
- Installed skills

## Step 1: Daily Local Backup via OS Cron

Set up a cron job on the **host machine** to pull the config directory from the pod daily.

Create `/usr/local/bin/openclaw-backup.sh`:

```bash
#!/bin/bash
set -euo pipefail

NAMESPACE="openclaw"
BACKUP_DIR="$HOME/openclaw-backups"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
DEST="$BACKUP_DIR/$TIMESTAMP"

POD=$(kubectl get pod -n "$NAMESPACE" \
  -l app.kubernetes.io/name=openclaw-helm \
  -o jsonpath='{.items[0].metadata.name}')

mkdir -p "$DEST"
kubectl cp "$NAMESPACE/$POD:/home/node/.openclaw" "$DEST"

# Retain last 7 days only
find "$BACKUP_DIR" -maxdepth 1 -type d -mtime +7 -exec rm -rf {} +

echo "Backup complete: $DEST"
```

Make it executable:

```bash
chmod +x /usr/local/bin/openclaw-backup.sh
```

Add to crontab (`crontab -e`):

```cron
0 2 * * * /usr/local/bin/openclaw-backup.sh >> /var/log/openclaw-backup.log 2>&1
```

This runs daily at 02:00 and retains the last 7 days of backups locally.

## Step 2: Sync to Remote Storage

We do not provide a specific sample here — the right solution depends on your infrastructure. Ask your AI assistant to generate a sync script tailored to your environment. Some common options:

| Destination | Tool | Example prompt |
|-------------|------|----------------|
| AWS S3 | `aws s3 sync` | "Write a script to sync `~/openclaw-backups` to S3 bucket `my-bucket` using aws cli" |
| Cloudflare R2 | `rclone` | "Write a rclone script to sync `~/openclaw-backups` to Cloudflare R2" |
| Another host | `rsync` over SSH | "Write a script to rsync `~/openclaw-backups` to `user@remote-host:/backups/openclaw`" |
| Google Drive / Dropbox | `rclone` | "Write a rclone script to sync `~/openclaw-backups` to my Google Drive folder" |

The sync step should run **after** the local backup in the same cron job or as a separate cron entry.

## Restore

To restore from a backup:

```bash
POD=$(kubectl get pod -n openclaw \
  -l app.kubernetes.io/name=openclaw-helm \
  -o jsonpath='{.items[0].metadata.name}')

kubectl cp ~/openclaw-backups/YYYYMMDD-HHMMSS/. openclaw/$POD:/home/node/.openclaw/
kubectl rollout restart deployment/openclaw-openclaw-helm -n openclaw
```

## Notes

- The Helm chart will **not** override an existing `openclaw.json` on the PVC, but always back up before upgrading.
- For k3s, prefix `kubectl` commands with `sudo -E` and set `KUBECONFIG=/etc/rancher/k3s/k3s.yaml`.
