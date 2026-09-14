# Bugbee architecture (original work: Superharness)

Upstream packages (TUI, desktop, providers, SDKs) are carried over from
[`anomalyco/opencode`](https://github.com/anomalyco/opencode) (`sst/opencode`
lineage) under MIT — see `NOTICE`. This document covers only the original
layer: `packages/bugbee/src/harness/` (~350 lines) + `docs/SUPERHARNESS.md`.

## Runtime position

```text
User
 ↓ prompt
Agent (build: full tools · plan: docs-only edits · review: read-only, max_steps 80)
 ↓ tool calls
Tool registry (edit/write/bash/read/glob/grep/web, output-bounded)
 ↓ tool.execute.after hook
HarnessPlugin (internal — no install step)
 ├─ trace? → append .bugbee/harness/trace.jsonl
 └─ mutating tool + verify enabled? → runVerify → inject report into tool output
 ↓
Execution (admit → drain → provider turn; process-local)
 ↓
Provider (BYOK)
```

Two plugin hooks, both in `plugin.ts`:

1. `tool.execute.after` — per tool call. Trace first (fire-and-forget,
   errors swallowed), then verify if the tool is in `verify.after_tools`
   (default `edit, write, apply_patch`) AND `verify.enabled` AND commands exist.
2. `experimental.chat.system.transform` — per turn. Pushes a short banner
   (verify commands, memory-dir pointer). Memory file *content* loads via
   `Instruction.systemPaths`, not here — this hook only points at the dir.

## Modules (`packages/bugbee/src/harness/`)

| File | Role |
|---|---|
| `types.ts` | `HarnessConfig` + per-feature configs, `TraceEvent`, defaults (80 steps, 4000 output chars, 120s timeout) |
| `config.ts` | `readHarness()` merges `harness` and `experimental.harness` (root wins); fills defaults: verify OFF, memory ON (`.bugbee/memory`), trace OFF |
| `plugin.ts` | Hook wiring above; per-session mutation counter; verify report appended to tool `output.output` + `harness_verify` metadata |
| `verify.ts` | `runVerify()`: sequential `spawn(shell:true)` per command, 200KB capture cap, SIGTERM→SIGKILL escalation, `[harness.verify]` text report with per-command exit/ms + clipped output |
| `memory.ts` | `loadMemoryFiles()`: ≤20 `.md/.txt`, sorted, 8KB/file cap; `ensureMemoryDir()` scaffolds dir + README |
| `trace.ts` | `appendTrace()`: mkdir + JSONL append, one line per tool step |

## Key properties

- **Opt-in verification.** `verify.enabled` defaults `false`: agents are fast by
  default, verifiable when the project opts in. No blocking gates — the report
  is injected into context and the model is told to fix failures.
- **Bounded context.** Every injection path is capped (output chars, file
  count/size, trace is disk-only, never auto-injected).
- **Process-local state.** Mutation counters and drains live in-process; no
  clustering. Trace is the cross-run memory (JSONL on disk).
- **Fail-open hooks.** Trace append failures are swallowed; config read
  failures fall back to defaults. A broken harness never breaks the agent loop.

## What this is NOT

Not a scheduler, not multi-tenant, not a sandbox (tool permissions come from
upstream), not an eval framework (no fixtures yet — see README future work).
