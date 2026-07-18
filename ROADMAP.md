# Roadmap

Aligned with [VISION.md](./VISION.md). Versions are intentional, not calendar promises.

## v0.1 — Foundation (current target)

- [x] Greenfield Rust workspace with stable crate boundaries
- [x] Domain types: findings, evidence, scores, review status, PoC, adjudication
- [x] Project config (`bugbee.toml`) + `bugbee init`
- [x] SQLite-backed local store + checkpoint resume
- [x] Secrets + YAML rule engine
- [x] Headless `hunt` / `findings` / `report` (SARIF + bounty)
- [x] OpenCode-style interactive workspace (slash commands)
- [x] Provider trait + OpenAI-compatible client with **tool calling**
- [x] Redaction helpers for outbound prompts
- [x] **Godmode harness**: tools, budgets, doom-loop
- [x] **Neuro-symbolic swarm**: Recon → Hunter → NSAE → Prover → Chain → Scribe
- [x] Attack Knowledge Graph (`bugbee-akg`)
- [x] NSAE matrix + static IR prover (`bugbee-nsae`)
- [x] Carlini refine loop + bounty ScribeAgent
- [x] `bugbee swarm` / `bugbee godmode` + TUI `/swarm`
- [x] CI workflow
- [ ] Installer script + release binaries

## v0.2 — Hunter quality

- [ ] Tree-sitter taint for Python / JS / TS
- [ ] WASM prover microVM (authorized targets only)
- [ ] OWASP Top 10 + India AppSec rule packs
- [ ] Fixture regression suite (DVWA/VAmPI offline packs)
- [ ] `bugbee connect` + OS keychain storage
- [ ] OpenAPI/GraphQL deep recon

## v0.3 — Agent depth

- [ ] Full tool loop: read, grep, edit (gated), shell (gated)
- [ ] Multi-role sessions (Scout → Taint → Reviewer → Patchsmith)
- [ ] Permission prompts in TUI
- [ ] Session resume + audit log export
- [ ] `/ask` grounded on repo index + findings

## v0.4 — Enterprise path

- [ ] Policy packs (enterprise template)
- [ ] Offline / air-gap mode docs + tests
- [ ] GitHub Action + SARIF upload path
- [ ] Signed releases + SLSA provenance (stretch)

## Later

- Language expansion (Go, PHP, Java, Rust self-analysis)
- Team queues and finding ownership
- Optional policy control plane
