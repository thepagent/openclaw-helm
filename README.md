# OpenClaw Helm Chart

A slim Helm chart for OpenClaw designed without Chromium browser integration. This chart focuses on minimal resource usage while maintaining full OpenClaw functionality for text-based interactions.

## Install

```bash
helm install openclaw oci://ghcr.io/thepagent/openclaw-helm -n openclaw --create-namespace
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

## Example: Add more skills

```yaml
skills:
  - weather
  - gog
  - your-skill-name
```
