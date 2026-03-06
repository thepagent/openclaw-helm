# OpenClaw Helm Chart

A slim Helm chart for OpenClaw designed without Chromium browser integration. This chart focuses on minimal resource usage while maintaining full OpenClaw functionality for text-based interactions.

## Install

```bash
helm install openclaw oci://ghcr.io/thepagent/openclaw-helm
```

## Design Philosophy

This chart is designed with three core principles:

1. **Minimal Resource Footprint** - Optimized to run on small cloud instances (2 vCPU, 2GB RAM). Tested on Zeabur and similar constrained environments.

2. **Decoupled Browser Architecture** - We exclude the Chromium sidecar container. Browser capabilities should be decoupled from the gateway. For browser automation, we recommend using Vercel [agent-browser](https://github.com/vercel-labs/agent-browser) with [Amazon Bedrock AgentCore Browser](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/browser-tool.html).

3. **Security by Design** - The gateway binds to loopback (127.0.0.1) even in Kubernetes environments. This ensures the gateway is only accessible through Kubernetes Service boundaries, not directly exposed on all network interfaces.

## Uninstall

```bash
helm uninstall openclaw
```

## Post-Installation Setup

After installing the chart, run the OpenClaw onboarding wizard to configure your AI provider:

```bash
kubectl exec -it deployment/openclaw-openclaw-helm -c main -- node dist/index.js onboard
```

This interactive wizard will guide you through:
- Security acknowledgment
- AI provider selection (OpenAI, Anthropic, etc.)
- Authentication setup (OAuth or API keys)
- Optional: Channel configuration (WhatsApp, Telegram, Slack, Discord, etc.)
- Optional: Skills installation

Your configuration will be stored in the pod's persistent volume.

### Alternative: API Key via Secret

If you prefer to skip the wizard and use API keys directly:

```bash
kubectl create secret generic openclaw-api-key \
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
POD=$(kubectl get pod -l app.kubernetes.io/name=openclaw-helm -o jsonpath='{.items[0].metadata.name}')
kubectl exec $POD -- openclaw models status
```

## Example: Add more skills

```yaml
skills:
  - weather
  - gog
  - your-skill-name
```
