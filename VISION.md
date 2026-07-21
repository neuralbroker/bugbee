# Bugbee — Product Vision

> **AI coding IDE with staff-level AppSec built in.**  
> Ship features. Hunt vulnerabilities. Prove them with evidence. Fix them safely.

## North star

Every engineering team should run one local agent that:

1. Writes and refactors code with best-in-class agent UX.
2. Hunts continuously with deterministic engines + multi-agent reasoning.
3. Proves every security claim with file/line/evidence — not vibes.
4. Patches with minimal, reviewable diffs under human or policy gates.
5. Exports audit-grade artifacts (SARIF, markdown) for CI and GRC.
6. Never exfiltrates secrets in the clear without redaction and policy.

If a pure coding agent is a pair programmer,  
Bugbee is **pair programmer + staff AppSec engineer** in one loop.

## Competitive moat

| Competitor pattern | Bugbee response |
|--------------------|-----------------|
| Coding agent only | Dual mandate: build + secure in one session |
| Scanner SaaS only | Local-first, agent-native triage and patch |
| LLM-only “find vulns” | Hybrid: deterministic rules/secrets first |
| Alert firehose | Evidence completeness + adversarial review agent |
| Cloud lock-in | BYOK, local models, offline engines |

## Product pillars

### 1. Hybrid intelligence

Engines find candidates cheaply and reproducibly. Agents correlate, prioritize, write patches, and challenge weak claims. A finding without evidence completeness stays draft.

### 2. Evidence-first claims

Every finding carries location, rule, severity, evidence trail, and status (`draft` → `confirmed` | `false_positive` | `fixed`).

### 3. Defense only

- No live exploitation modules.
- No weaponized payload generation against unauthorized targets.
- PoCs stay local, synthetic, educational.
- Network tools remain opt-in, logged, and permission-gated.

### 4. Sovereignty

- BYOK and local models (Ollama / air-gapped OpenAI-compatible gateways)
- Redaction of secrets in tool output
- Offline engines for pure hunts; LLM optional
- CI-exportable SARIF/JSON

## Roadmap themes

See [ROADMAP.md](./ROADMAP.md).

## Lineage

Forked from [OpenCode](https://github.com/anomalyco/opencode) (MIT) for agent runtime and UX excellence; rebranded and specialized into **Bugbee** as a distinct product. Not affiliated with the OpenCode team.
