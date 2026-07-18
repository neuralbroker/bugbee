# BugBee AI Agent Executable Specification (Implementation Map)

**Version:** 0.1.0-alpha · **Runtime:** Rust · **Paradigm:** Neuro-Symbolic Autonomous Vulnerability Discovery  

Defense-only. Authorized codebases and local fixtures. No live third-party exploitation.

## Architecture → crates

| Spec component | Crate / module |
|----------------|----------------|
| Domain models (Finding, PoC, Target, NSAE types) | `bugbee-core` |
| Attack Knowledge Graph | `bugbee-akg` |
| Neuro-Symbolic Adjudication Engine | `bugbee-nsae` |
| Swarm + godmode harness + tools | `bugbee-agent` |
| Deterministic scanners | `bugbee-engine` |
| LLM tool-calling | `bugbee-llm` |
| TUI | `bugbee-ui` |
| CLI | `bugbee` |

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

## CLI

```bash
bugbee swarm -v                         # full neuro-symbolic pipeline
bugbee swarm --report out.md
bugbee swarm --no-resume
bugbee godmode --offline                # engines + enrich (+ LLM agents if connected)
bugbee report --format bounty -o r.md
```

## Success criteria status

| Criterion | Status |
|-----------|--------|
| Local scan without internet | ✅ engines + swarm offline |
| Checkpoint resume | ✅ `.bugbee/checkpoint.json` |
| Adjudication filters noise | ✅ matrix + prover |
| Bounty-formatted report | ✅ ScribeAgent |
| Verified PoC pack (local) | ✅ static IR + steps |
| WASM sandbox / live HTTP prover | ⏳ Horizon B |
| Edge 1.5B distilled model | ⏳ Horizon D |
| Tree-sitter multi-lang taint | ⏳ Horizon B |

## Anti-replication moats (in code)

1. **Verification gap** — findings pass prover IR before “verified”
2. **Adjudication matrix** — asymmetric symbolic/neural rules
3. **AKG** — causal chains, not flat finding lists
4. **Rust single-binary core** — memory-safe, air-gap capable
5. **Auth-aware recon stubs** — surface mapping before hunt
