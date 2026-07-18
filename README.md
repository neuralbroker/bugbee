<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/license-Apache%202.0-blue" alt="License">
  <img src="https://img.shields.io/badge/tests-124-green?logo=github" alt="Tests">
  <img src="https://img.shields.io/badge/clippy-0%20warnings-brightgreen" alt="Clippy">
  <img src="https://img.shields.io/badge/PRs-welcome-purple" alt="PRs">
</p>

<h1 align="center">🐝 Bugbee</h1>

<p align="center">
  <strong>Your terminal security engineer.</strong><br>
  Find vulnerabilities. Prove them with evidence. Fix them safely.
</p>

<p align="center">
  <code>cargo install --path crates/bugbee --locked</code> ·
  <code>bugbee init</code> ·
  <code>bugbee hunt</code>
</p>

<p align="center">
  <i>Terminal-first · Memory-safe (Rust) · Deterministic engines · BYOK models · Human-first review</i>
</p>

---

## ✨ Features at a glance

| | |
|---|---|
| 🔍 **Smart hunts** | Deterministic rules + secrets engine + multi-agent LLM swarm |
| 🧠 **AI-powered** | BYOK — bring your own key (OpenAI, Anthropic, Ollama, xAI, 10+ providers) |
| 🧵 **Kill chains** | Attack Knowledge Graph links findings into exploit paths |
| 🛡️ **Safe by design** | No live exploits · secrets redacted before outbound calls · defense only |
| 🖥️ **Interactive TUI** | Full workspace with slash commands, finding review, live events |
| 📦 **Single binary** | `cargo install` · air-gap ready · no Node/Python runtime |
| 📊 **SARIF export** | CI-ready · bounty reports · compliance packs |
| 🧪 **124 tests** | CLI integration, crawler, kill-chain, engine, harness, E2E validation |

---

## 🚀 Get started in 30 seconds

```bash
# 1. Install
cargo install --path crates/bugbee --locked

# 2. Initialize in your project
cd /your/repo
bugbee init

# 3. Run a hunt
bugbee hunt

# 4. See what it found
bugbee findings

# 5. Export a report
bugbee report --format bounty -o report.md
```

That's it. No accounts, no cloud, no configuration needed. Engines work **offline** — the LLM is optional.

---

## 🎮 Commands

```bash
bugbee                    # Launch interactive TUI workspace
bugbee init               # Set up bugbee.toml + local state
bugbee hunt               # Deterministic rules + secrets scan
bugbee findings           # List all findings
bugbee review <id> confirm|fp|fixed   # Human review
bugbee report --format bounty -o r.md # Markdown report
bugbee report --output results.sarif.json  # SARIF for CI
bugbee doctor             # Check configuration readiness
bugbee connect --provider ollama --model qwen2.5-coder  # Add LLM
bugbee ask "What are the top risks?"   # Ask AI about your repo
bugbee swarm -v           # Full neuro-symbolic pipeline
bugbee godmode            # Multi-phase AI harness
bugbee super "goal"       # SuperHarness agent loop
```

### 👀 TUI workspace

| Key | Action |
|-----|--------|
| `/hunt` or `h` | Run local hunt |
| `/findings` | Browse findings |
| `/review <id> confirm\|fp\|fixed` | Human review |
| `c` / `f` / `x` | Confirm / false-positive / fixed |
| `Tab` | Switch hunt ↔ review role |
| `/doctor` | Config check |
| `/report` | Export SARIF |
| `q` | Quit |

---

## 🔌 Connect an AI model (optional but powerful)

```bash
# OpenAI
bugbee connect --provider openai --model gpt-4o
export OPENAI_API_KEY=sk-...

# Anthropic Claude
bugbee connect --provider anthropic --model claude-3-5-haiku-latest
export ANTHROPIC_API_KEY=...

# Ollama (local, free)
bugbee connect --provider ollama --model qwen2.5-coder
# No API key needed — runs on your machine

# Or any of 10+ providers:
# xAI · Kimi · DeepSeek · OpenRouter · HuggingFace · Z.ai · GLM · custom
```

When connected, Bugbee becomes an **AI security engineer** — it reads files, runs targeted hunts, correlates findings, and writes evidence-backed reports. No model? No problem — the deterministic engines work fully offline.

---

## 🏗️ Architecture

```
┌─────────────┐  bugbee          CLI entry point
├─────────────┤  bugbee-ui       Terminal UI (Ratatui)
├─────────────┤  bugbee-agent    Swarm · harness · tools · crawl · hunt
├─────────────┤  bugbee-nsae     Neuro-symbolic adjudication + prover
├─────────────┤  bugbee-akg      Attack Knowledge Graph (kill chains)
├─────────────┤  bugbee-engine   Rules · secrets · sandbox
├─────────────┤  bugbee-llm      BYOK providers (tool-calling)
├─────────────┤  bugbee-core     Types · store · config · scope · redaction
└─────────────┘  bugbee-harness  gRPC Super Harness (diff oracle, verify)
```

**Dependency rule:** `ui → agent → {engine, llm, harness} → core`<br>
Engine never calls the network. Secrets are redacted before outbound LLM calls.

### Key modules

| Module | What it does |
|--------|-------------|
| **Hunter** | Parses LLM vulnerability hypotheses, mock provider for testing |
| **Crawler** | HTTP crawler with robots.txt, cycle detection, API discovery |
| **Super Harness** | gRPC Unix-socket verification server + differential oracle |
| **Authorization** | HARD RULE: `--i-have-permission` flag + scope file for live targets |
| **AKG** | Attack Knowledge Graph — kill chains, pivot suggestions, topology |
| **VS Code** | Sidebar extension — import SARIF, navigate findings, verify |

---

## 🔒 Security & trust

- **Defense only.** No exploitation modules. No weaponized payloads.
- **HARD RULE auth gate.** Live-target commands require `--i-have-permission` + scope file.
- **Redacted by default.** Secrets stripped before any outbound LLM call.
- **Sensitive paths blocked.** Tool reads respect policy blocks.
- **Audit trail.** All findings carry evidence, location, and status history.
- **Rust single binary.** Memory-safe, reproducible builds.

---

## 📚 Learn more

| Doc | What's inside |
|-----|---------------|
| [SPEC.md](./SPEC.md) | Full implementation specification with success criteria |
| [HARNESS.md](./HARNESS.md) | SuperHarness architecture & integration map |
| [VISION.md](./VISION.md) | Product constitution & long-term north star |
| [ROADMAP.md](./ROADMAP.md) | Versioned milestones & future horizons |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | How to build, test, and send PRs |
| [SECURITY.md](./SECURITY.md) | Reporting vulnerabilities in Bugbee itself |

---

## 🧪 Test suite

```bash
cargo test --workspace   # 124 tests, all passing
cargo clippy --workspace # 0 warnings
```

---

<p align="center">
  Built with 🦀 Rust & ❤️ for AppSec engineers everywhere.<br>
  <i>Bugbee is inspired by <a href="https://opencode.ai">OpenCode</a> but is an independent project — not affiliated with the OpenCode team.</i>
</p>

<p align="center">
  <a href="https://github.com/neuralbroker/bugbee">GitHub</a> ·
  <a href="./LICENSE">Apache 2.0</a>
</p>
