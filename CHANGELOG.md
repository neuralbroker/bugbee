# Changelog

All notable changes are documented here.

## Unreleased

- OpenCode-style interactive workspace: `bugbee` (no args) with slash commands (`/hunt`, `/findings`, `/review`, `/doctor`, `/ask`).
- India AppSec rule pack for gov/edu/BFSI/enterprise hygiene (CERT-In oriented), plus expanded OWASP web rules.
- Aggressive hunt mode (default): lower drop threshold, multi-pack loading, PHP/Java indexing and taint.
- Secrets coverage for India payment-gateway style credentials (redacted evidence only).
- SpaceXAI-inspired product site under `site/` (Next.js, Vercel-ready).
- Harness security corpus (1980s→2026 defensive technique map) + classics/modern rule pack + `security_playbook` tool.
- Readiness fixes: TUI nested-runtime panic, `--no-india`/`--no-aggressive`, terminal restore on panic.

## 0.1.0-beta.1 — 2026-07-10

First public beta of Bugbee.

- Terminal-first deterministic bug and vulnerability hunts for Python, JavaScript, TypeScript, and Go.
- Embedded OWASP-focused baseline rules, secrets detection, scope-limited taint heuristics, SARIF export, and a local SQLite review queue.
- Local-first agent safeguards: sensitive-path restrictions, redaction before model-bound content, permission-gated tools, and human review by default.
- Cross-platform release archives for Linux, macOS (Intel and Apple Silicon), and Windows.
- Production hardening for beta: precompiled detectors, redacted evidence storage, thread-safe SQLite store with WAL, unambiguous finding-id resolution, and expanded secret patterns.
- One-line installer for macOS/Linux: `curl -fsSL https://github.com/neuralbroker/bugbee/releases/latest/download/install-bugbee.sh | bash`.

### Beta notes

- Bugbee is defensive security tooling. Validate findings and patches in your own environment before production use.
- Native provider adapters are still evolving; OpenAI-compatible endpoints and local Ollama-style endpoints are the current supported model transport.
