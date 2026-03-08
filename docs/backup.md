# Backup

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

## Restore

To restore from a backup:

1. Copy the backup directory back into the pod with `kubectl cp`
2. Restart the deployment with `kubectl rollout restart`

## Notes

- The Helm chart will **not** override an existing `openclaw.json` on the PVC, but always back up before upgrading.
- For k3s, prefix `kubectl` commands with `sudo -E` and set `KUBECONFIG=/etc/rancher/k3s/k3s.yaml`.
