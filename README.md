# Bugbee

> **Attribution:** derived from [anomalyco/opencode](https://github.com/anomalyco/opencode) (`sst/opencode` lineage) under MIT — see [NOTICE](./NOTICE). Original work here is the **Superharness** layer (`packages/bugbee/src/harness/`, `docs/SUPERHARNESS.md`). Upstream packages are carried over, not original.

AI coding agent with durable runs and a reviewable trail.

## Problem

Agents lose project context, tool history, permissions, and verification state across real-repo work.

## Solution

A permission-scoped agent loop with explicit state and opt-in verification:

```text
User
 ↓
Agent (build/plan/review)
 ↓
Tools (edit/write/bash/read/glob/grep/web)
 ↓
Execution (admit → drain → provider turn)
 ↓
Verification (verify-after-mutate + doctor)
 ↓
Project Context (memory + trace + compaction)
```

## Architecture

1. **Agent:** `build` (full tools), `plan` (docs-only edits), `review` (read-only). Steps capped (`max_steps` 80).
2. **Tools:** confined filesystem/code/web set gated by `allow/ask/deny`.
3. **State:** prompt admission + process-local drain; `.bugbee/memory/*.md` loaded as instructions; compaction starts a new epoch.
4. **Verify:** `harness.verify.commands` (e.g. `bun test`) run after mutations; `trace.jsonl` appended; `bugbee doctor` for offline health.

Details: `docs/SUPERHARNESS.md`, `CONTEXT.md`, `AGENTS.md`.

## Key Engineering Decisions

1. Permission-scoped modes over one omnipotent agent.
2. Explicit admission/drain semantics over implicit concurrency.
3. File-based memory + trace over hidden state.
4. Opt-in post-mutation verification over blocking gates.

## Tech Stack

TypeScript · Bun · SQLite (Drizzle) · Effect

## Features

1. Terminal/TUI/desktop/IDE/API surfaces
2. Permission-scoped agent modes
3. Tool registry with output bounding
4. Durable session/message/part/todo tables
5. Memory + trace + compaction epochs
6. Verify-after-mutate + doctor checks

## Running Locally

```bash
bun install --ignore-scripts
bun run dev            # or ./bin/bugbee
./bin/bugbee doctor
```

Requires Bun 1.3+.

## Testing

```bash
bun test --timeout 30000   # 600+ tests across packages (upstream + harness)
```

## Performance

No published agent benchmarks in this repo. Do not cite success-rate/latency numbers.

## Limitations

Monorepo carries full upstream SST/TUI/Electron/Nix surface; single squashed import (no upstream sync); verification is opt-in (`enabled: false` by default); drains are process-local (no clustering).

## Future Improvements

Extract Superharness as a standalone plugin with eval fixtures; publish agent success/verify metrics only after measurement. Details: `docs/ARCHITECTURE.md`, `docs/CONFIG.md`, `docs/SUPERHARNESS.md`.
