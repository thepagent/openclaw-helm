# OpenClaw Helm Chart

A slim Helm chart for OpenClaw designed without Chromium browser integration. This chart focuses on minimal resource usage while maintaining full OpenClaw functionality for text-based interactions.

## Install

```bash
helm install openclaw oci://ghcr.io/thepagent/openclaw-helm --version 1.0.1
```

## Design Philosophy

This chart is intentionally slim and excludes the Chromium browser sidecar container to:
- Reduce memory and CPU usage
- Simplify deployment in resource-constrained environments
- Focus on text-based AI agent capabilities

If you need browser automation features, consider the official OpenClaw chart or enable browser support by modifying `config.browser.enabled` in values.yaml.

## Uninstall

```bash
helm uninstall openclaw-custom
```

## Configuration

Edit `values.yaml` to customize:

- `image.tag` - OpenClaw version
- `config.gateway.controlUi.allowedOrigins` - CORS origins
- `config.browser.enabled` - Enable/disable browser (default: false)
- `skills` - List of skills to install
- `resources` - CPU/memory limits

## AI Provider Authentication

**This chart deploys OpenClaw without pre-configured AI providers.** After installation, authenticate with your AI provider:

### Option 1: OpenAI OAuth (Recommended)

```bash
# Get pod name
POD=$(kubectl get pod -l app.kubernetes.io/name=openclaw-helm -o jsonpath='{.items[0].metadata.name}')

# Run interactive OAuth login
kubectl exec -it $POD -- openclaw models auth login --provider openai-codex
```

Follow the prompts to complete OAuth authentication. Your credentials are stored securely in the pod's persistent volume.

### Option 2: API Key (Alternative)

Create a Kubernetes secret with your API key:

```bash
kubectl create secret generic openclaw-api-key \
  --from-literal=OPENAI_API_KEY=sk-...
```

Then add to your values.yaml:

```yaml
envFrom:
  - secretRef:
      name: openclaw-api-key
```

### Verify Authentication

```bash
kubectl exec $POD -- openclaw models status
```

## Example: Add more skills

```yaml
skills:
  - weather
  - gog
  - your-skill-name
```
