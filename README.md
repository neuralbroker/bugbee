# Bugbee

**Terminal-first, memory-safe security agent for vulnerability hunting.**

Lightweight TUI/CLI · deterministic local engines · model-agnostic BYOK · human review.

> Defensive only. No live exploitation. Built for enterprises and confidential codebases.<br>
> Written in **Rust** for memory safety and a single static binary.

Agent UX is **inspired by** [OpenCode](https://opencode.ai) (slash commands, hunt/review roles like build/plan).<br>
Bugbee is an independent project and is **not affiliated** with the OpenCode team.

---

## Long-term vision

See **[VISION.md](./VISION.md)** for the product constitution and **[ROADMAP.md](./ROADMAP.md)** for horizons.

In short: every engineering org should run one local binary that understands their repo like a staff AppSec engineer — proves findings with evidence, patches safely, and never ships untrusted guesswork as “critical.”

## Install (from source)

```bash
cargo install --path crates/bugbee --locked
bugbee --version
```

Or run from a checkout:

```bash
cargo build --release -p bugbee
./target/release/bugbee --help
```

## Quick start

```bash
cd /path/to/your/project
bugbee init
bugbee swarm -v         # neuro-symbolic multi-agent swarm (recommended)
bugbee super -v "…"     # SuperHarness tool loop (Pi+OpenCode+Claude)
bugbee godmode          # godmode pipeline (uses SuperHarness when LLM on)
bugbee                  # interactive workspace (/swarm /godmode /hunt)
# engines only:
bugbee hunt
bugbee findings
bugbee report --format bounty -o report.md
bugbee report --output findings.sarif.json
```

### Godmode harness (OpenCode-style tools, security niche)

```
Phase 0  ENGINE     rules + secrets (git-aware walk)
Phase 1  ENRICH     parallel context windows (rayon)
Phase 2  HUNT LEAD  LLM tool loop (read/grep/glob/hunt/findings/…)
Phase 3  TAINT      deep evidence on critical/high
Phase 4  REVIEW     adversarial FP killer
Phase 5  REPORT     digest + store
```

Tools exposed to the model: `hunt`, `read`, `grep`, `glob`, `list_dir`,
`list_findings`, `get_finding`, `review_finding`, `add_evidence`, `todo_write`
(plus gated `shell`). Budget + doom-loop protection included.

```bash
bugbee godmode              # uses LLM if connected
bugbee godmode --offline    # engines + enrich only
bugbee godmode -v           # print harness events
```

### Interactive workspace

| Input | Action |
|-------|--------|
| `/godmode` or `g` | Offline godmode pipeline |
| `/hunt` or `h` | Deterministic local hunt |
| `/findings` | Refresh finding list |
| `/review <id> confirm\|fp\|fixed` | Human review |
| `c` / `f` / `x` | Confirm / false-positive / fixed |
| `Tab` | Switch hunt ↔ review role |
| `/doctor` | Config readiness |
| `/report` | Write SARIF |
| `q` | Quit |

### Connect a model (optional)

```bash
bugbee connect --provider xai --model grok-4.5 --api-key-env XAI_API_KEY
export XAI_API_KEY=...
bugbee ask "What are the top risks in this repo?"
```

Supports OpenAI, native Anthropic Messages API, xAI, Kimi, Z.ai/GLM, DeepSeek,
OpenRouter, Ollama, Hugging Face Inference Providers, and custom OpenAI-compatible
gateways. API keys are read only from environment variables.

```bash
# Native Anthropic tool calling (Messages API)
bugbee connect --provider anthropic --model claude-3-5-haiku-latest
export ANTHROPIC_API_KEY=...

# OpenAI-compatible routes
bugbee connect --provider kimi --model kimi-k2.5
export MOONSHOT_API_KEY=...

bugbee connect --provider hf --model openai/gpt-oss-120b:fastest
export HF_TOKEN=...
```

## Architecture

```
bugbee          CLI entry
bugbee-ui       Ratatui workspace
bugbee-agent    Swarm · godmode harness · tools · permissions
bugbee-nsae     Neuro-symbolic adjudication + static prover
bugbee-akg      Attack Knowledge Graph (kill chains)
bugbee-engine   Rules · secrets
bugbee-llm      BYOK providers (tool calling)
bugbee-core     Types · config · SQLite · NSAE matrix · PoC · redaction
```

Full agent specification: **[SPEC.md](./SPEC.md)** · SuperHarness analysis: **[HARNESS.md](./HARNESS.md)** · vision: **[VISION.md](./VISION.md)**

## Security

- Policy `defense_only` cannot be disabled
- Secrets redacted before outbound LLM calls
- Sensitive paths blocked from tool reads by default
- Report product issues via [SECURITY.md](./SECURITY.md)

## License

Apache-2.0 — see [LICENSE](./LICENSE).
