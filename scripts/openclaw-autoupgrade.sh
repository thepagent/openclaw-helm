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
