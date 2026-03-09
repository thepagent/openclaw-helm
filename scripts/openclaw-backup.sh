#!/usr/bin/env bash
# openclaw-backup.sh — Layer 1 + Layer 2 backup using `openclaw backup create`
# Requires OpenClaw >= 2026.3.8
set -euo pipefail

NAMESPACE="openclaw"
BACKUP_DIR="${BACKUP_DIR:-$HOME/Backups/openclaw}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"
# Optional: set REMOTE_DEST to enable Layer 2 sync (e.g. s3://bucket/openclaw-backups)
REMOTE_DEST="${REMOTE_DEST:-}"

# Auto-detect: on k3s host use sudo + k3s kubeconfig; otherwise use KUBECONFIG env
if [[ -f /etc/rancher/k3s/k3s.yaml ]]; then
  KUBECTL="sudo KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl"
else
  KUBECTL="kubectl"
fi

# Resolve running pod
POD=$($KUBECTL get pod -n "$NAMESPACE" -l app.kubernetes.io/name=openclaw-helm \
  --field-selector=status.phase=Running -o jsonpath='{.items[0].metadata.name}')

echo "[backup] pod: $POD"
mkdir -p "$BACKUP_DIR"

# Layer 1 — run backup inside pod, copy archive out
RESULT=$($KUBECTL exec -n "$NAMESPACE" "$POD" -- \
  openclaw backup create --output /tmp --verify --json)

ARCHIVE=$(echo "$RESULT" | grep '"archivePath"' | head -1 | sed 's/.*"archivePath": *"\([^"]*\)".*/\1/')
FILENAME=$(basename "$ARCHIVE")

echo "[backup] archive: $ARCHIVE"
$KUBECTL cp "${NAMESPACE}/${POD}:${ARCHIVE}" "${BACKUP_DIR}/${FILENAME}"
echo "[backup] saved: ${BACKUP_DIR}/${FILENAME}"

# Prune old backups
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +"$RETENTION_DAYS" -delete
echo "[backup] pruned backups older than ${RETENTION_DAYS} days"

# Layer 2 — remote sync (optional)
if [[ -n "$REMOTE_DEST" ]]; then
  aws s3 sync "$BACKUP_DIR/" "$REMOTE_DEST/"
  echo "[backup] synced to $REMOTE_DEST"
fi

echo "[backup] done"
