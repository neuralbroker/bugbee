<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75%2B-orange" alt="Rust">
  <img src="https://img.shields.io/badge/license-Apache%202.0-blue" alt="License">
  <img src="https://img.shields.io/badge/tests-124-green" alt="Tests">
  <img src="https://img.shields.io/badge/clippy-0%20warnings-brightgreen" alt="Clippy">
  <img src="https://img.shields.io/badge/PRs-welcome-purple" alt="PRs">
</p>

<h1 align="center">Bugbee</h1>

<p align="center">
  <strong>Terminal-native security engineering.</strong><br>
  Discover vulnerabilities. Prove them with evidence. Resolve them safely.
</p>

<p align="center">
  <code>cargo install --path crates/bugbee --locked</code> ·
  <code>bugbee init</code> ·
  <code>bugbee hunt</code>
</p>

<p align="center">
  <i>Memory-safe (Rust) · Deterministic engines · BYOK models · Terminal-first · Human-in-the-loop review</i>
</p>

---

## Features

| | |
|---|---|
| **Smart hunts** | Deterministic rules engine + secrets scanning + multi-agent LLM swarm |
| **AI-powered** | Bring your own key (OpenAI, Anthropic, Ollama, xAI, 10+ providers) |
| **Kill chains** | Attack Knowledge Graph links individual findings into exploitable attack paths |
| **Safe by design** | No live exploits · secrets redacted before outbound calls · defense-only tooling |
| **Interactive TUI** | Full workspace with slash commands, finding review, live streaming events |
| **Single binary** | `cargo install` · air-gap ready · no Node.js or Python runtime required |
| **SARIF export** | CI-ready output · bounty reports · compliance pack generation |
| **124 tests** | CLI integration, crawler, kill-chain topology, engine, harness, E2E validation |

---

## Quick start

```bash
# Install
cargo install --path crates/bugbee --locked

# Initialize in your project
cd /your/repo
bugbee init

# Run a hunt
bugbee hunt

# Review findings
bugbee findings

# Export a report
bugbee report --format bounty -o report.md
```

No accounts, no cloud services, no configuration required. All deterministic engines work fully offline — the LLM is optional.

---

## Commands

```bash
bugbee                    # Launch interactive TUI workspace
bugbee init               # Configure bugbee.toml and local state
bugbee hunt               # Deterministic rules + secrets scan
bugbee findings           # List all findings
bugbee review <id> confirm|fp|fixed   # Human review
bugbee report --format bounty -o r.md # Markdown report
bugbee report --output results.sarif.json  # SARIF for CI
bugbee doctor             # Check configuration readiness
bugbee connect --provider ollama --model qwen2.5-coder  # Add LLM
bugbee ask "What are the top risks?"   # Query AI about your repository
bugbee swarm -v           # Full neuro-symbolic pipeline
bugbee godmode            # Multi-phase AI harness
bugbee super "goal"       # SuperHarness agent loop
```

### TUI workspace

| Key | Action |
|-----|--------|
| `/hunt` or `h` | Run local hunt |
| `/findings` | Browse findings |
| `/review <id> confirm\|fp\|fixed` | Human review |
| `c` / `f` / `x` | Confirm / false-positive / fixed |
| `Tab` | Switch between hunt and review roles |
| `/doctor` | Configuration check |
| `/report` | Export SARIF report |
| `q` | Quit |

---

## Connect an AI model (optional)

```bash
# OpenAI
bugbee connect --provider openai --model gpt-4o
export OPENAI_API_KEY=sk-...

# Anthropic Claude
bugbee connect --provider anthropic --model claude-3-5-haiku-latest
export ANTHROPIC_API_KEY=...

# Ollama (local, free)
bugbee connect --provider ollama --model qwen2.5-coder
# No API key required — runs entirely on your machine

# Other supported providers:
# xAI · Kimi · DeepSeek · OpenRouter · HuggingFace · Z.ai · GLM · custom
```

When connected, Bugbee operates as an AI security engineer — it reads files, runs targeted hunts, correlates findings across modules, and writes evidence-backed reports. Without a model, all deterministic engines function fully offline.

---

## Architecture

```
bugbee              CLI entry point
bugbee-ui           Terminal UI (Ratatui)
bugbee-agent        Swarm orchestration · harness · tools · crawler · hunter
bugbee-nsae         Neuro-symbolic adjudication and prover
bugbee-akg          Attack Knowledge Graph (kill chain topology)
bugbee-engine       Rules engine · secrets detection · sandbox
bugbee-llm          BYOK provider abstraction with tool-calling support
bugbee-core         Types · store · config · scope enforcement · redaction
bugbee-harness      gRPC Super Harness (differential oracle, verification)
```

**Dependency rule:** `ui -> agent -> {engine, llm, harness} -> core`

The engine never calls the network. Secrets are redacted before any outbound LLM call.

### Key modules

| Module | Description |
|--------|-------------|
| **Hunter** | Parses LLM vulnerability hypotheses; mock provider for testing |
| **Crawler** | HTTP crawler with robots.txt, cycle detection, API discovery |
| **Super Harness** | gRPC Unix-socket verification server with differential oracle |
| **Authorization** | Hard-rule enforcement: `--i-have-permission` flag + scope file for live targets |
| **AKG** | Attack Knowledge Graph — kill chains, pivot suggestions, topology analysis |
| **VS Code** | Sidebar extension for SARIF import, finding navigation, and verification |

---

## Security and trust

- **Defense only.** No exploitation modules. No weaponized payloads.
- **Authorization gate.** Live-target commands require `--i-have-permission` and a scope file.
- **Redacted by default.** Secrets and credentials are stripped before any outbound LLM call.
- **Sensitive path protection.** Tool reads respect policy-blocked paths.
- **Audit trail.** All findings carry evidence, location, and status history.
- **Rust single binary.** Memory-safe, reproducible builds.

---

## Documentation

| Document | Contents |
|----------|----------|
| [SPEC.md](./SPEC.md) | Full implementation specification with success criteria |
| [HARNESS.md](./HARNESS.md) | SuperHarness architecture and integration map |
| [VISION.md](./VISION.md) | Product constitution and long-term direction |
| [ROADMAP.md](./ROADMAP.md) | Versioned milestones and future horizons |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | Build, test, and contribution guide |
| [SECURITY.md](./SECURITY.md) | Reporting vulnerabilities in Bugbee itself |

---

## Test suite

```bash
cargo test --workspace   # 124 tests, all passing
cargo clippy --workspace # 0 warnings
```

---

<p align="center">
  <i>Bugbee is inspired by <a href="https://opencode.ai">OpenCode</a> but is an independent project — not affiliated with the OpenCode team.</i>
</p>

<p align="center">
  <a href="https://github.com/neuralbroker/bugbee">GitHub</a> ·
  <a href="./LICENSE">Apache 2.0</a>
</p>
