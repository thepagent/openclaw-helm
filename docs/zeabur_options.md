# Deploying OpenClaw on Zeabur: Template vs Helm Chart

[Zeabur](https://zeabur.com/) has become one of the popular VPS providers, offering a curated collection of self-baked one-click templates as well as K3s-ready instances out of the box. This makes it a natural fit for running OpenClaw — and it offers two distinct ways to do so. This document compares them to help you choose.

## Options at a Glance

| | Zeabur Template | openclaw-helm on Zeabur K3s |
|---|---|---|
| OpenClaw version | `2026.3.8` (pinned in template) | `2026.3.13` (latest) |
| Upgrade | Change image tag in Zeabur Dashboard | `helm upgrade openclaw oci://ghcr.io/thepagent/openclaw-helm` |
| Gateway bind | LAN (publicly reachable) | loopback (K8s Service boundary only) |
| Service exposure | HTTP port exposed | `ClusterIP` — not directly reachable from outside |
| Chromium | Optional sidecar | Excluded (smaller attack surface) |
| `dangerouslyDisableDeviceAuth` | Enabled (required for cloud) | Disabled (device pairing retained) |
| Credentials | Zeabur environment variables | K8s Secret (`helm.sh/resource-policy: keep`) |
| External secret managers | ❌ | ✅ via External Secrets Operator (Vault, AWS SM, GCP SM) |
| `redactSensitive` | Not set | `tools` (sensitive data in tool calls is masked) |
| Config management | Edit `openclaw.json` via Zeabur UI | Declarative `values.yaml` |
| Skills auto-install | Manual | initContainer runs `clawhub install` on startup |
| Health checks | Auto-restart wrapper script | K8s liveness / readiness / startup probes (3 layers) |
| Backup | `openclaw backup create` CLI | `kubectl cp` or Velero |
| Data sovereignty | Data on Zeabur platform | On-prem / private cloud possible |
| Zero-config deploy | ✅ One-click | Requires `helm install` |

## When to Use the Zeabur Template

- You want OpenClaw running in under 5 minutes with zero Kubernetes knowledge.
- You are doing a personal PoC or sharing a trial link with others.
- You do not have strict security or data sovereignty requirements.

## When to Use openclaw-helm on Zeabur K3s

Zeabur's built-in K3s means you can run the Helm chart on the same platform without managing your own cluster. This gives you the best of both worlds: Zeabur's managed infrastructure (networking, storage, domain) combined with the Helm chart's stricter security posture.

Choose openclaw-helm when you:

- Want the latest OpenClaw version and one-line upgrades.
- Need loopback-bound gateway + ClusterIP isolation.
- Need to integrate with enterprise secret managers (Vault, AWS Secrets Manager) via External Secrets Operator.
- Require data sovereignty (GDPR, financial regulations, on-prem).
- Use [agent-browser](https://github.com/vercel-labs/agent-browser) with [Amazon Bedrock AgentCore Browser](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/browser-tool.html) — the decoupled browser architecture aligns perfectly.

### Recommended Setup

```bash
# Install on Zeabur's K3s
helm install openclaw oci://ghcr.io/thepagent/openclaw-helm \
  -n openclaw --create-namespace

# Upgrade to latest chart + image in one command
helm upgrade openclaw oci://ghcr.io/thepagent/openclaw-helm
```

The gateway token Secret is created automatically and retained on uninstall (`helm.sh/resource-policy: keep`), so paired clients survive upgrades without re-pairing.

For API keys, use a K8s Secret and reference it via `envFrom`:

```bash
kubectl create secret generic openclaw-api-keys \
  --from-literal=OPENAI_API_KEY=sk-... \
  -n openclaw
```

```yaml
# values.yaml
envFrom:
  - secretRef:
      name: openclaw-api-keys
```

## Limitations of Both Options

Neither option currently provides:

- SSO / LDAP / OIDC enterprise identity integration
- High availability (`replicas: 1` in both)
- Audit logging for compliance

For enterprise production use, these gaps should be addressed before adoption.

## Wrapping Up

- **Just want it running fast?** Use the Zeabur Template — one click, zero setup.
- **On Zeabur with K3s?** Skip the Template and use `helm install` instead. You get the latest version, loopback security, and one-line upgrades with no extra infrastructure cost.
- **Self-hosted / on-prem / enterprise PoC?** openclaw-helm is the only viable path — it works on any K8s cluster and integrates with enterprise secret managers.
- **Enterprise production?** Both options need SSO, HA, and audit logging before they are production-ready. openclaw-helm's open architecture makes those gaps easier to fill.

**Bottom line:** If you have access to any K8s environment (including Zeabur's built-in K3s), openclaw-helm is the better choice in every dimension except initial setup simplicity.

---

*This document was assessed by [Kiro](https://kiro.dev) using Claude Opus 4.6 on March 14, 2026.*
