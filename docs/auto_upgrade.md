# Auto-Upgrade

> **Note:** This document is intended to be read together with your AI agent. Share it directly and ask your agent to implement the scripts for your specific environment.

An auto-upgrade checker runs on a cron schedule, compares the latest OCI registry version against the currently deployed version, and only proceeds when a new version is detected — backing up first, then upgrading, and sending a notification on success.

## Sample Script

```bash
#!/bin/bash
set -e

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
NAMESPACE="openclaw"
RELEASE_NAME="openclaw"
CHART="oci://ghcr.io/thepagent/openclaw-helm"

CURRENT=$(sudo -E helm list -n $NAMESPACE -o json | jq -r ".[] | select(.name==\"$RELEASE_NAME\") | .chart" | sed 's/openclaw-helm-//')
LATEST=$(sudo -E helm show chart $CHART | grep '^version:' | awk '{print $2}')

if [ "$CURRENT" = "$LATEST" ]; then
    echo "Already up to date: $CURRENT"
    exit 0
fi

echo "Upgrade available: $CURRENT → $LATEST"

/home/ubuntu/openclaw-backup.sh

sudo -E helm upgrade $RELEASE_NAME $CHART -n $NAMESPACE

sudo -E kubectl rollout status deployment/openclaw-openclaw-helm -n $NAMESPACE --timeout=120s

POD_NAME=$(sudo -E kubectl get pod -n $NAMESPACE -l app.kubernetes.io/name=openclaw-helm -o jsonpath='{.items[0].metadata.name}')
sudo -E kubectl exec -n $NAMESPACE $POD_NAME -- openclaw message send --channel telegram --target <YOUR_TELEGRAM_CHAT_ID> -m "✅ OpenClaw upgraded: $CURRENT → $LATEST"

echo "Upgraded: $CURRENT → $LATEST"
```

Save this as `/usr/local/bin/openclaw-autoupgrade.sh`, make it executable, then schedule it:

```cron
0 * * * * /usr/local/bin/openclaw-autoupgrade.sh >> /var/log/openclaw-autoupgrade.log 2>&1
```

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
