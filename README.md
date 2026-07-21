<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
  <img src="https://img.shields.io/badge/runtime-Bun-black" alt="Bun">
  <img src="https://img.shields.io/badge/agents-coding%20%2B%20security-purple" alt="Agents">
  <img src="https://img.shields.io/badge/defense--only-yes-green" alt="Defense only">
</p>

<h1 align="center">Bugbee</h1>

<p align="center">
  <strong>The AI coding IDE that also hunts vulnerabilities.</strong><br>
  Build software. Prove risks with evidence. Patch safely.
</p>

<p align="center">
  <code>bun install</code> ·
  <code>bun run dev</code> ·
  <code>bugbee hunt</code> ·
  <code>/hunt</code>
</p>

<p align="center">
  <i>Terminal · Desktop · IDE · Deterministic scanners · BYOK models · Human-in-the-loop</i>
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="VISION.md">Vision</a> ·
  <a href="ROADMAP.md">Roadmap</a> ·
  <a href="SECURITY.md">Security</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

## Why Bugbee

Most AI coding agents stop at “write the feature.”  
Most AppSec tools stop at “here’s a PDF of alerts.”

**Bugbee is both:** an agent runtime with OpenCode-class UX, specialized for **security-native development**.

| Mode | What you get |
|------|----------------|
| **Build** | Pair-programming agent with LSP, multi-agent tasks, plan mode, permissions |
| **Hunt** | Deterministic secrets + OWASP-style rule scanners → evidence store |
| **Review** | Adversarial read-only subagent that kills weak findings |
| **Report** | Markdown reports + SARIF 2.1.0 for CI |

Hard product law: **defense only**. No live exploitation modules. No weaponized payloads against third-party systems. Fixtures and PoCs stay local, synthetic, and educational.

---

## Quick start

### Prerequisites

- [Bun](https://bun.sh) 1.3+
- A model provider key when using agentic mode (Anthropic, OpenAI, Ollama, xAI, …)

### Install & run (from source)

```bash
git clone https://github.com/neuralbroker/bugbee.git
cd bugbee
bun install --ignore-scripts   # full monorepo; --ignore-scripts avoids optional native rebuild issues

# Local CLI wrapper (preserves your cwd for relative paths)
./bin/bugbee --help

# Interactive TUI
bun run dev
# or:
./bin/bugbee
```

### Offline security hunt (no LLM required)

```bash
# Scan a directory with secrets + rule engines
./bin/bugbee hunt /path/to/project
# or from repo root:
bun run hunt ./fixtures/python-vuln

# List findings
./bin/bugbee findings --directory ./fixtures/python-vuln

# SARIF for CI
./bin/bugbee hunt ./fixtures/python-vuln --format sarif -o bugbee-results.sarif.json

# Markdown report
./bin/bugbee hunt ./fixtures/python-vuln --format markdown -o bugbee-report.md
```

Try the intentional fixture:

```bash
./bin/bugbee hunt ./fixtures/python-vuln
```

### Interactive agent (with model)

```bash
bun run dev
```

Inside a project session:

```text
/hunt                 # defensive hunt: scanners + evidence triage
/findings             # list & triage open findings
/report               # markdown or SARIF export
```

Or chat:

```text
Hunt this repo for injection and secrets, then patch the top confirmed issues.
```

Config: `bugbee.json` / `bugbee.jsonc` and `.bugbee/`.  
Findings store: `.bugbee/findings.json`.  
XDG paths: `~/.config/bugbee`, `~/.local/share/bugbee`, `~/.cache/bugbee`.

---

## CLI surface

| Command | Purpose |
|---------|---------|
| `bugbee` | Interactive TUI (default) |
| `bugbee hunt [dir]` | Offline deterministic scan (secrets + rules) |
| `bugbee findings` | List / update finding status |
| `bugbee run …` | Non-interactive agent turn |
| `bugbee agent list` | List agents (`build`, `hunt`, `security-review`, …) |
| `bugbee providers` | Configure BYOK credentials |
| `bugbee serve` / `web` | Headless server / web UI |

---

## Security tools (agent-facing)

| Tool | Purpose |
|------|---------|
| `secrets_scan` | High-signal credential detectors (values redacted) |
| `vuln_scan` | OWASP-inspired heuristics (injection sinks, weak crypto, XSS, …) |
| `findings` | `list` / `get` / `set_status` (`draft` → `confirmed` \| `false_positive` \| `fixed`) |
| `security_report` | Markdown or SARIF 2.1.0 export |

### Agents

| Agent | Role |
|-------|------|
| `build` | Default — coding + defensive security dual mandate |
| `plan` | Plan mode (no edits) |
| `hunt` | Security hunt lead (primary) |
| `security-review` | Read-only adversarial reviewer (subagent) |
| `explore` / `general` | Codebase exploration / parallel tasks |

---

## Architecture

```
packages/
  bugbee/     CLI, TUI entry, tools, security engine
  core/       Agent runtime, providers, permissions, plugins
  app/        Shared web UI
  desktop/    Desktop shell
  tui/ ui/    Terminal & shared components
  sdk/ schema/  Public types & client SDK
  plugin/     Plugin surface
```

Security engine (`packages/bugbee/src/security/`):

```
rules + secrets → scanner → .bugbee/findings.json → SARIF / markdown
         ↑                           ↑
  vuln_scan / secrets_scan    findings / security_report / bugbee hunt
```

Deterministic engines run fully offline. LLMs are optional for correlation, patches, and explanation.

Self-test:

```bash
bun packages/bugbee/src/security/selftest.ts
```

---

## Differentiation

1. **One agent for ship + secure** — no context switch between a coding IDE and a separate scanner SaaS.
2. **Evidence completeness** — findings without path/line/snippet stay draft; adversarial review is first-class.
3. **Deterministic + LLM hybrid** — cheap scanners first; models for dataflow and patches, not inventing CVEs.
4. **Defense-only constitution** — safe for enterprise source and regulated environments.
5. **BYOK / local models** — no forced cloud for core hunts.
6. **CI-native artifacts** — SARIF + store under `.bugbee/` for PR gates.
7. **Proven agent UX lineage** — OpenCode-class interaction model, specialized into a distinct product.

---

## Configuration

```jsonc
// bugbee.jsonc
{
  "$schema": "https://bugbee.dev/config.json",
  "model": "anthropic/claude-sonnet-4-5"
  // "default_agent": "hunt"
}
```

Environment flags use the `BUGBEE_*` prefix (see `packages/core/src/flag/flag.ts`).

---

## Development

```bash
bun install --ignore-scripts
bun run dev                          # TUI
./bin/bugbee hunt ./fixtures/python-vuln
bun run test:security                # security engine selftest
bun run test:bugbee                  # packages/bugbee unit suite
bun run typecheck                    # turbo typecheck (when workspaces fully installed)
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) and [AGENTS.md](./AGENTS.md).

---

## Documentation map

| Doc | Contents |
|-----|----------|
| [VISION.md](./VISION.md) | North star, moat, constitution |
| [ROADMAP.md](./ROADMAP.md) | Near-term product plan |
| [SECURITY.md](./SECURITY.md) | Product posture + vulnerability reporting |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | How to contribute |
| [AGENTS.md](./AGENTS.md) | Conventions for human and AI contributors |
| [NOTICE](./NOTICE) | Upstream OpenCode attribution |
| [LICENSE](./LICENSE) | MIT |

Localized READMEs (`README.*.md`) link back to this English product README.

---

## Lineage & license

Bugbee is a **product fork** of [OpenCode](https://github.com/anomalyco/opencode) (MIT).  
Upstream credit belongs to the OpenCode authors and contributors.  
Bugbee is an independent project and is **not affiliated** with the OpenCode team.

Licensed under the **MIT License** — see [LICENSE](./LICENSE) and [NOTICE](./NOTICE).

---

## Security policy

See [SECURITY.md](./SECURITY.md). Report product vulnerabilities privately.  
**Do not** use Bugbee to attack systems you do not own or lack authorization to test.
