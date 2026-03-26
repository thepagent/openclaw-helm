# OpenClaw Helm Chart

A slim Helm chart for OpenClaw designed without Chromium browser integration. This chart focuses on minimal resource usage while maintaining full OpenClaw functionality for text-based interactions.

## Install

There are two ways to install this chart.

### OCI Registry（recommended）

Pull directly from GitHub Container Registry without any repo setup:

```bash
helm install openclaw oci://ghcr.io/thepagent/openclaw-helm \
  -n openclaw --create-namespace
```

Install a specific version:

```bash
helm install openclaw oci://ghcr.io/thepagent/openclaw-helm \
  --version 1.3.11 \
  -n openclaw --create-namespace
```

Upgrade to a newer version:

```bash
helm upgrade openclaw oci://ghcr.io/thepagent/openclaw-helm -n openclaw
```

### Helm Repository（GitHub Pages）

Add the Helm repository and install from there:

```bash
helm repo add openclaw https://thepagent.github.io/openclaw-helm
helm repo update

# Install latest
helm install openclaw openclaw/openclaw-helm -n openclaw --create-namespace

# Install a specific version
helm install openclaw openclaw/openclaw-helm --version 1.3.11 -n openclaw --create-namespace
```

Browse available versions:

```bash
helm search repo openclaw --versions
```

The `--create-namespace` flag will create the `openclaw` namespace if it doesn't exist.

### Verify Installation

Run the built-in connectivity test to verify the gateway is responding:

```bash
helm test openclaw -n openclaw
```

This validates that the OpenClaw gateway is accessible on localhost within the pod.

## Design Philosophy

This chart is designed with three core principles:

1. **Minimal Resource Footprint** - Optimized to run on small cloud instances (2 vCPU, 2GB RAM). Tested on Zeabur and similar constrained environments.

2. **Decoupled Browser Architecture** - We exclude the Chromium sidecar container. Browser capabilities should be decoupled from the gateway. For browser automation, we recommend using Vercel [agent-browser](https://github.com/vercel-labs/agent-browser) with [Amazon Bedrock AgentCore Browser](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/browser-tool.html).

3. **Security by Design** - The gateway binds to loopback (127.0.0.1) even in Kubernetes environments. This ensures the gateway is only accessible through Kubernetes Service boundaries, not directly exposed on all network interfaces.

## Post-Installation Setup

After installing the chart, run the OpenClaw onboarding wizard to configure your AI provider:

```bash
sudo kubectl exec -it deployment/openclaw-openclaw-helm -n openclaw -c main -- openclaw onboard
```

This interactive wizard will guide you through:
- Security acknowledgment
- AI provider selection (OpenAI, Anthropic, etc.)
- Authentication setup (OAuth or API keys)
- Optional: Channel configuration (WhatsApp, Telegram, Slack, Discord, etc.)
- Optional: Skills installation

Your configuration will be stored in the pod's persistent volume.

### Configure Channels (e.g. Telegram)

To configure channels after onboarding:

```bash
# Enter the container
sudo kubectl exec -it deployment/openclaw-openclaw-helm -n openclaw -c main -- openclaw configure --section channels

# Approve a pairing request
sudo kubectl exec -it deployment/openclaw-openclaw-helm -n openclaw -c main -- openclaw pairing approve telegram *******
```

### Alternative: API Key via Secret

If you prefer to skip the wizard and use API keys directly:

```bash
sudo kubectl create secret generic openclaw-api-key \
  --from-literal=OPENAI_API_KEY=sk-...
```

Update your values.yaml:

```yaml
envFrom:
  - secretRef:
      name: openclaw-api-key
```

Upgrade the release:

```bash
helm upgrade openclaw oci://ghcr.io/thepagent/openclaw-helm -f values.yaml
```

### Verify Setup

```bash
POD=$(sudo kubectl get pod -n openclaw -l app.kubernetes.io/name=openclaw-helm -o jsonpath='{.items[0].metadata.name}')
sudo kubectl exec -n openclaw $POD -- openclaw models status
```

## Mount Host `~/.openclaw` into the Pod

```
Host (k3s node)
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ┌─────────────────┐   ln -sf   ┌──────────────────────────┐   │
│  │  ~/.openclaw    │ ──────────▶│  /var/lib/rancher/k3s/   │   │
│  │  (symlink)      │            │  storage/pvc-xxx/        │   │
│  └─────────────────┘            └────────────┬─────────────┘   │
│                                              │ (physical dir)  │
└──────────────────────────────────────────────┼─────────────────┘
                                               │ mounted via PVC
                                               │
                          ┌────────────────────▼─────────────────┐
                          │              Pod                      │
                          │  ┌─────────────────────────────────┐  │
                          │  │  /home/node/.openclaw           │  │
                          │  │  (PVC mount)                    │  │
                          │  └─────────────────────────────────┘  │
                          │  ┌─────────────────────────────────┐  │
                          │  │  openclaw gateway               │  │
                          │  └─────────────────────────────────┘  │
                          └───────────────────────────────────────┘

  host ~/.openclaw  ══════════════  pod /home/node/.openclaw
               (same physical directory, both R/W)
```

If you are running on a single-node setup (e.g. k3s with `local-path` StorageClass), you can symlink your host `~/.openclaw` to the PVC data directory so the pod and host share the same config, sessions, and skills.

```bash
# Find the PVC's actual host path
PV_PATH=$(kubectl get pv \
  $(kubectl get pvc openclaw-openclaw-helm -n openclaw -o jsonpath='{.spec.volumeName}') \
  -o jsonpath='{.spec.hostPath.path}')

echo "PVC is at: $PV_PATH"

# Symlink ~/.openclaw on the host to the PVC path
ln -sf "$PV_PATH" ~/.openclaw
```

> **Note:** This works only with local StorageClasses (e.g. k3s `local-path`) where the PV has a real host path. It does not apply to NFS, EBS, or other remote storage backends.

## Backup

Before upgrading or making changes, backup your OpenClaw configuration:

```bash
POD=$(sudo kubectl get pod -n openclaw -l app.kubernetes.io/name=openclaw-helm -o jsonpath='{.items[0].metadata.name}')
sudo kubectl cp openclaw/$POD:/home/node/.openclaw ~/openclaw-backup-$(date +%Y%m%d-%H%M%S)
```

This backs up all configuration, credentials, and channel settings to your host machine.

**Note:** This Helm chart will NOT override `openclaw.json` if it already exists in the PVC, but it's always best practice to backup before upgrading.

### Restore from Backup

To restore a backup:

```bash
POD=$(sudo kubectl get pod -n openclaw -l app.kubernetes.io/name=openclaw-helm -o jsonpath='{.items[0].metadata.name}')
sudo kubectl cp ~/openclaw-backup-YYYYMMDD-HHMMSS/. openclaw/$POD:/home/node/.openclaw/
sudo kubectl rollout restart deployment/openclaw-openclaw-helm -n openclaw
```

## Upgrade

To upgrade to the latest version:

```bash
helm upgrade openclaw oci://ghcr.io/thepagent/openclaw-helm -n openclaw
```

**For k3s users:** You need to specify the kubeconfig path:

```bash
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
sudo -E helm upgrade openclaw oci://ghcr.io/thepagent/openclaw-helm -n openclaw
```

The `-E` flag preserves the `KUBECONFIG` environment variable when running with sudo.

## Uninstall

```bash
helm uninstall openclaw
```

## Further Reading

To customize your own backup or auto-upgrade strategy, read:
- [docs/backup.md](docs/backup.md) — backup concept, workflow, and restore steps
- [docs/auto_upgrade.md](docs/auto_upgrade.md) — automated version checking and upgrade workflow

Share these docs with your AI agent and ask it to implement the scripts for your environment.

## Pod Scheduling

Use standard Kubernetes scheduling fields to control where the OpenClaw pod runs:

```yaml
nodeSelector:
  disktype: ssd

tolerations:
  - key: "dedicated"
    operator: "Equal"
    value: "openclaw"
    effect: "NoSchedule"

affinity:
  nodeAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      nodeSelectorTerms:
        - matchExpressions:
            - key: topology.kubernetes.io/zone
              operator: In
              values:
                - us-east-1a
```

## Example: Add more skills

```yaml
skills:
  - weather
  - gog
  - your-skill-name
```
