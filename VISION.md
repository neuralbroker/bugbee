# Bugbee — Long-Term Vision

> **Terminal-first, memory-safe security agent.**  
> Find real vulnerabilities. Prove them with evidence. Fix them safely. Never exploit live systems.

Bugbee is not a general coding agent with a security plugin.  
It is a **security-native agent runtime** — built like [OpenCode](https://opencode.ai) for interaction and agent UX, but purpose-built for **defensive vulnerability research** on codebases you own or are authorized to assess.

---

## North star (10-year)

Every engineering org should be able to run one local binary that:

1. **Understands** their repository with the same depth an expert AppSec engineer would.
2. **Hunts** continuously with deterministic engines + multi-agent reasoning.
3. **Proves** every claim with file/line/dataflow evidence — not vibes.
4. **Patches** with minimal, reviewable diffs under human or policy gate.
5. **Exports** audit-grade artifacts (SARIF, evidence packs) for compliance and CI.
6. **Never leaves** confidential source in the clear to untrusted clouds without redaction and explicit policy.

If OpenCode is "pair programmer in the terminal,"  
Bugbee is **"staff AppSec engineer + SRE of trust, in the terminal."**

---

## Why Rust (and only memory-safe systems languages)

Security tools that touch enterprise source code must be trustworthy themselves.

| Principle | Why it matters |
|-----------|----------------|
| **Memory safety by default** | No class of buffer/UAF bugs in the agent that holds your secrets and source |
| **Single static binary** | Easy air-gap / regulated install; no Node supply-chain for the core runtime |
| **Fearless concurrency** | Parallel hunts, indexing, and provider I/O without data races |
| **Auditability** | Small, reviewable crates; clear ownership of redaction and policy paths |
| **Longevity** | A 10-year product needs a language that ages with systems software, not framework churn |

**Policy:** Core runtime, scanners, redaction, store, and TUI stay in **Rust**.  
Optional language adapters (tree-sitter grammars, language-specific taint) may use safe FFI or pure data packs — never require a second runtime for the agent core.

---

## Product pillars

### 1. Hybrid intelligence (deterministic + LLM)

```
┌──────────────┐     ┌─────────────────┐     ┌──────────────────┐
│  Index & AST │────▶│ Local engines   │────▶│ Agent reasoning  │
│  (ignore-    │     │ rules · secrets │     │ evidence · patch │
│   aware)     │     │ taint · sinks   │     │ multi-role review│
└──────────────┘     └─────────────────┘     └──────────────────┘
```

- **Engines** find candidates cheaply and reproducibly.
- **Agents** correlate, prioritize, write patches, and challenge weak claims.
- A finding without **evidence completeness** does not leave the draft queue.

### 2. OpenCode-grade agent UX

Borrow the interaction model users already love; specialize the domain:

| OpenCode idea | Bugbee equivalent |
|---------------|-------------------|
| `build` / `plan` agents | `hunt` (active) / `review` (read-only adversarial) |
| Tab switch agents | Switch roles: Scout · Taint · Reviewer · Patchsmith |
| Tool permissions | Path blocks, secret redaction, no-network-by-default for exploits |
| Slash commands | `/hunt` `/findings` `/review` `/ask` `/report` `/doctor` |
| Sessions | Hunt sessions with audit log + resume |

We are **inspired by** OpenCode's agent UX. Bugbee is an independent project and is **not affiliated** with the OpenCode team.

### 3. Evidence-first security claims

Every finding carries:

- **Location** — path, range, symbol
- **Trace** — source → sink (when taint applies)
- **Rule / agent** — who claimed it
- **Scores** — Bugbee Risk Score (BRS) + Evidence Completeness (ECS)
- **Status** — draft → confirmed | false_positive | fixed

No silent "high severity" without a trail a human can re-verify offline.

### 4. Defense only

Hard product law:

- No live exploitation modules.
- No weaponized payload generation against third-party systems.
- Fixtures and PoCs stay **local, synthetic, and educational**.
- Network tools (if any) are opt-in, logged, and policy-gated.

### 5. Enterprise & sovereignty

- **BYOK** and local models (Ollama / air-gapped OpenAI-compatible gateways)
- **Redaction** before any outbound LLM call
- **Policy packs** (enterprise, India AppSec / CERT-In oriented, custom YAML)
- **SARIF + JSON** for CI, GRC, and bug trackers
- Works offline for pure engine hunts; LLM optional

---

## Architecture (stable crate boundaries)

```
bugbee          CLI + process entry
bugbee-ui       Ratatui workspace (presentation only)
bugbee-agent    Sessions, tools, permissions, multi-agent loop, crawl, hunter
bugbee-engine   Deterministic scanners & rule packs
bugbee-llm      Provider-agnostic model clients (BYOK)
bugbee-core     Types, config, store, scoring, redaction, errors, scope
bugbee-akg      Attack Knowledge Graph (kill chains, topology)
bugbee-harness  gRPC Super Harness (Unix socket, diff oracle, verification)
```

**Dependency rule:** `ui → agent → {engine, llm, harness} → core`.  
UI never talks to providers or the store except through agent/core APIs.  
Engine never calls the network.

---

## Multi-agent constitution (long-term roles)

| Role | Mandate |
|------|---------|
| **Hunt Lead** | Plans coverage, prioritizes attack surface |
| **Scout** | Fast breadth: configs, secrets, dependency smells |
| **Taint Analyst** | Deep dataflow source→sink proofs |
| **Adversarial Reviewer** | Tries to kill weak findings (read-only) |
| **Patchsmith** | Minimal fixes + tests; never silent behavior change |

Roles may run as sequential phases or concurrent workers under a shared session and permission envelope.

---

## Non-goals

- Replacing general-purpose coding agents for feature development
- Offensive C2, mass scanning of the public internet, or "auto-pwn"
- Hosting customer source in our cloud as a requirement
- Rewriting the core in a GC / JIT language for convenience

---

## Success metrics

1. **Precision** — confirmed findings that survive human review  
2. **Recall on fixtures** — known vulns always re-found  
3. **Time-to-evidence** — seconds for engine path; minutes for agent path  
4. **Trust** — zero critical CVEs in Bugbee itself; reproducible builds  
5. **Adoption** — single-binary install, works on air-gapped laptops  

---

## Guiding aphorisms

1. **If you can't show the line, you don't have a finding.**  
2. **The agent is untrusted; the evidence store is not.**  
3. **Speed without memory safety is someone else's incident.**  
4. **Fix the bug; don't become the bug.**  

---

*This document is the product constitution. Implementation PRs that contradict it require an explicit VISION amendment.*
