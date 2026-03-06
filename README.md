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

OpenClaw supports OAuth-based onboarding for AI providers (OpenAI, Anthropic, etc.). Users authenticate through the OpenClaw UI rather than providing API keys directly.

For development/testing, you can optionally provide API keys via environment variables:

```yaml
env:
  ANTHROPIC_API_KEY: "sk-ant-..."
```

Or use Kubernetes secrets:

```bash
kubectl create secret generic openclaw-secrets \
  --from-literal=ANTHROPIC_API_KEY=sk-ant-...
```

## Example: Add more skills

```yaml
skills:
  - weather
  - gog
  - your-skill-name
```
