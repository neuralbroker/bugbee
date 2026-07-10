<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/brand/bugbee-mark-light.png">
    <img src="assets/brand/bugbee-mark-v2.png" width="144" alt="Bugbee logo — an abstract geometric signal-path mark">
  </picture>
</p>

<h1 align="center">Bugbee</h1>

<p align="center">
  <strong>Terminal-first agentic IDE for bug fixing and vulnerability hunting.</strong><br>
  Lightweight TUI/CLI · heavy local analysis · model-agnostic BYOK · human + auto review.
</p>

> Defensive only. No live exploitation. Designed for enterprises and confidential codebases.

## Features

- **Any model**: OpenAI-compatible endpoints, xAI Grok, DeepSeek, Qwen, Kimi, GLM, Claude, GPT, Ollama, OpenRouter, custom gateways — you bring the key and model id
- **Deterministic engines**: pattern rules, secrets, taint heuristics (Python / JS / TS / Go)
- **Scoring**: Bugbee Risk Score (BRS) + Evidence Completeness (ECS) + dual-review gates
- **Review queue**: confirm / false-positive / fixed with SARIF export
- **Redaction**: secrets scrubbed before LLM calls; sensitive paths blocked
- **Agent constitution**: hunt lead, scout, taint analyst, adversarial reviewer, patchsmith

## Quick start

```bash
# from this repo
cargo build --release -p bugbee-cli
export PATH="$PWD/target/release:$PATH"

cd /path/to/your/project
bugbee init
# optional: any provider + any model
bugbee connect --provider xai --api-key "$XAI_API_KEY" --model grok-4.5
# or local
bugbee connect --provider ollama --base-url http://127.0.0.1:11434/v1 --model qwen2.5-coder
bugbee doctor

bugbee hunt
bugbee findings
bugbee tui
bugbee report --output findings.sarif.json
```

Keys passed to `bugbee connect --api-key` are stored in the local OS keychain;
they are not written to `bugbee.toml`. Environment variables remain supported
for CI and headless environments.

### Demo fixtures

```bash
bugbee --root fixtures/python-vuln init
bugbee --root fixtures/python-vuln hunt
bugbee --root fixtures/python-vuln findings
```

## CLI

| Command | Description |
|---------|-------------|
| `bugbee init` | Create `bugbee.toml`, `BUGBEE.md`, `.bugbee/` |
| `bugbee connect` | BYOK — configure any provider/model |
| `bugbee hunt` | Index + engines (+ optional `--llm-review`) |
| `bugbee findings` | List by BRS |
| `bugbee review <id> confirm\|fp\|fixed` | Human review |
| `bugbee report` | SARIF export |
| `bugbee ask "..."` | Chat with configured model about the repo |
| `bugbee models` | List providers / remote model ids |
| `bugbee doctor` | Validate configuration and provider readiness without network calls |
| `bugbee tui` | Terminal UI |

## Architecture

```
bugbee (CLI/TUI)
    └── core engines + harness (local)
            ├── index (repo map)
            ├── rules / taint / secrets
            ├── finding store (SQLite) + BRS/ECS
            └── providers (any model via OpenAI-compat + keys)
```

## Config (`bugbee.toml`)

```toml
[inference]
# ANY model id your endpoint accepts
hunt = "provider/model"
scout = "provider/model"
review = "other-provider/model"   # prefer different vendor for review

[providers.my-custom]
base_url = "https://your-gateway/v1"
api_key_env = "MY_KEY"
protocol = "openai_compat"
```

## Security product boundary

Bugbee is a **defensive AppSec assistant**. It will not help attack third-party systems. Sandbox network defaults to deny. Patches require human approval workflows.

## License

Apache-2.0. Enterprise services and commercial offerings may be distributed
under separate terms.
