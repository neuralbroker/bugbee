# Bugbee AI Agent Executable Specification (Implementation Map)

**Version:** 0.1.0-alpha · **Runtime:** Rust · **Paradigm:** Neuro-Symbolic Autonomous Vulnerability Discovery  

Defense-only. Authorized codebases and local fixtures. No live third-party exploitation.

## Architecture → crates

| Spec component | Crate / module |
|----------------|----------------|
| Domain models (Finding, PoC, Target, NSAE types) | `bugbee-core` |
| Authorization gate (scope, `--i-have-permission`) | `bugbee-core::scope` |
| Attack Knowledge Graph | `bugbee-akg` |
| Neuro-Symbolic Adjudication Engine | `bugbee-nsae` |
| Swarm + godmode harness + tools + hunter + crawl | `bugbee-agent` |
| Deterministic scanners + sandbox | `bugbee-engine` |
| gRPC Super Harness (server, client, diff oracle) | `bugbee-harness` |
| LLM tool-calling | `bugbee-llm` |
| TUI | `bugbee-ui` |
| CLI | `bugbee` |
| VS Code extension | `vscode/` |

## Swarm topology (implemented)

```
ReconAgent → HunterAgent (engines + Carlini)
          → SymbolicAgent + NSAE matrix
          → ProverAgent (static IR verification)
          → ChainAgent (AKG kill chains)
          → ScribeAgent (bounty markdown)
```

Shared memory: findings store + AKG + `.bugbee/checkpoint.json`.

## NSAE pipeline

1. **AST structural extract** — sources/sinks/guards (`bugbee-nsae::ast`)
2. **Neural hypothesis** — calibrated proxy (+ optional LLM later)
3. **Deterministic IR translator** — formal checks (not raw Lean from LLM)
4. **Adjudication matrix** — LeanGuard-style asymmetric rules in `bugbee-core::adjudicate`

## Prover

MVP: **StaticProver** executes IR against local source (pattern, guards, secret assign).  
Horizon: wasmtime microVM + authorized target mirror (not started).

## Hunter Agent

Located in `crates/bugbee-agent/src/hunter.rs`:

- `parse_hypotheses()` — parses LLM JSON into structured `Hypothesis` objects, gracefully degrades on malformed input
- `hypotheses_to_findings()` — converts hypotheses to `Finding` objects with PoCs
- `build_hunt_prompt()` — generates endpoint analysis prompts for LLMs
- `MockLlmClient` — test double implementing `LlmClient` with pre-configured responses

## Crawler

Located in `crates/bugbee-agent/src/crawl.rs`:

- BFS crawler with depth/max-URL limits
- `robots.txt` parsing and path disallow enforcement
- Cycle detection via visited set
- HTML link extraction (anchors + form actions)
- API spec discovery (OpenAPI, Swagger, GraphQL, AsyncAPI well-known paths)
- URL normalization (fragment strip, trailing slash, whitespace)

## Authorization Gate

Located in `crates/bugbee-core/src/scope.rs`:

**HARD RULE:** All live-target commands (hunt, verify, replay) must refuse unless `--i-have-permission` flag AND scope file both authorize the target; enforced first, before any hunting logic.

- `url_in_scope()` — checks URL against allowed hosts (globbed) and URL prefixes
- `load_scope()` — loads TOML/YAML/JSON scope files
- 18 tests proving refusal without permission flag, out-of-scope hosts/IPs, empty scope

## CLI

```bash
bugbee swarm -v                         # full neuro-symbolic pipeline
bugbee swarm --report out.md
bugbee swarm --no-resume
bugbee godmode --offline                # engines + enrich (+ LLM agents if connected)
bugbee report --format bounty -o r.md
bugbee findings                         # list findings
bugbee review <id> confirm|fp|fixed     # human review
bugbee doctor                           # config readiness
bugbee connect --provider ollama ...    # configure provider
bugbee super -v "goal"                  # SuperHarness agent loop
```

## VS Code Extension

Located in `vscode/`:

- Webview sidebar showing severity-sorted findings with color-coded badges
- SARIF import/export via commands
- Click-to-navigate to finding locations
- Verification toggle per finding
- Summary bar (total, critical, high, verified counts)

## Success criteria status

| Criterion | Status |
|-----------|--------|
| Local scan without internet | ✅ engines + swarm offline |
| Checkpoint resume | ✅ `.bugbee/checkpoint.json` |
| Adjudication filters noise | ✅ matrix + prover |
| Bounty-formatted report | ✅ ScribeAgent |
| Verified PoC pack (local) | ✅ static IR + steps |
| Authorization gate HARD RULE | ✅ scope + `--i-have-permission` |
| gRPC Super Harness | ✅ Unix-socket, diff oracle, typed errors |
| LLM hypothesis parsing | ✅ `parseHypotheses` + mock provider |
| HTTP crawler | ✅ robots.txt, cycle detection, API spec discovery |
| AKG kill chains | ✅ topology, difficulty scoring, pivot suggestions |
| VS Code sidebar | ✅ import/export, navigation, verification |
| E2E validation against fixture | ✅ `fixtures/python-vuln/` |
| WASM sandbox / live HTTP prover | ⏳ Horizon B |
| Edge 1.5B distilled model | ⏳ Horizon D |
| Tree-sitter multi-lang taint | ⏳ Horizon B |

## Anti-replication moats (in code)

1. **Verification gap** — findings pass prover IR before "verified"
2. **Adjudication matrix** — asymmetric symbolic/neural rules
3. **AKG** — causal chains, not flat finding lists
4. **Rust single-binary core** — memory-safe, air-gap capable
5. **Auth-aware recon stubs** — surface mapping before hunt
6. **Authorization gate** — impossible to bypass scope checking
7. **DiffOracle** — time-delay threshold, content-change ratio, error-leak detection
