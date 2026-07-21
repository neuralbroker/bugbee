# Superharness research notes

Minimal notes from public forums, OSS coding agents, and harness literature.
Goal: evolve Bugbee’s agent runtime beyond a plain ReAct loop without marketing fluff.

## User signals (Reddit / X, 2025–2026)

Recurring wants:

| Need | Why it matters |
|------|----------------|
| Keep multi-file project context | Agents lose track on large codebases |
| Short feedback loops | IDE/terminal integration beats paste chat |
| Control over autonomy | Overconfident agents edit too much |
| BYOK / cost control | Subscriptions and credits spike |
| Switchable roles | plan vs build vs review in one thread |
| Diff review before accept | Second-pass review catches bad changes |
| Multi-session / worktrees | Parallel tasks without context mash |
| Persistent memory / skills | Don’t re-teach every session |

Sources include r/AI_Agents, r/vibecoding, r/GithubCopilot discussions and X posts comparing OpenCode, Claude Code, Codex, Cursor.

## Harness = Model + control plane

Consensus definition (see arXiv:2606.10106 “What makes a harness a harness”):

1. **Agent loop** — plan → act → observe  
2. **Tool interface** — edit, bash, search, MCP  
3. **Context management** — compaction, files, memory  
4. **Control** — permissions, hooks, stop conditions  

Fowler-style: *Agent = Model + Harness*. The model is commodity; the harness is product.

## OSS reference patterns

| System | Useful pattern for Bugbee |
|--------|---------------------------|
| **Claude Code** | Strong project memory (`CLAUDE.md`), hooks (pre/post tool), permission allow/deny, session logs |
| **Codex CLI** | Tight sandbox defaults; long-running batch style tasks |
| **OpenCode (upstream)** | Dual plan/build agents, BYOK, provider switch, permissions |
| **Cline** | Multi-editor surface; explicit approval on edits |
| **Aider / OpenHands / SWE-agent** | Benchmark-driven loops, repo-level evaluation |
| **Cursor** | Fast context + agent mode; users still want less surprise edits |

## Superharness modules (proposed)

Ship in this order — each module independently useful and testable.

### M1 — Loop integrity (foundation)
- Hard caps: max steps, doom-loop detection (already partial)
- Structured tool results + truncation paths
- Explicit “done” criteria (tests green, checklist)

### M2 — Context engine
- Anchored compaction (keep goals, files, decisions)
- Session memory file under `.bugbee/memory/` (optional)
- Skill packs portable across agents

### M3 — Control plane
- Permission matrix (edit/bash/network) with session-scoped allows
- Pre/post tool hooks (format, lint, secret scan)
- Human gates on write/network by default policy packs

### M4 — Multi-agent orchestration
- Role agents: plan (RO), build, review (adversarial RO)
- Parallel explore workers with merge report
- Parent/child sessions already exist — formalize contracts

### M5 — Verify loop (“super” layer)
- After edit batch: run project tests / typecheck when available
- Auto-open review summary (diff + test output)
- Optional second-model critique before marking task complete

### M6 — Observability
- Per-step trace (tool, path, tokens, latency)
- Export run as JSON for evals
- Offline doctor already checks install health

## What not to build first

- Live exploit / attack automation  
- Loud marketing surfaces  
- Another full IDE fork before harness quality is proven  

## Shipped in codebase

| Module | Status | Where |
|--------|--------|--------|
| M1 Loop integrity | shipped | `doom_loop` permission; default `agent.steps` via `harness.max_steps` (80) |
| M2 Context / memory | shipped | `.bugbee/memory/*.md` loaded as instructions when enabled |
| M3 Control hooks | shipped | Internal `HarnessPlugin` on `tool.execute.after` |
| M4 Multi-agent | shipped | native `review` subagent (read-only) |
| M5 Verify | shipped | opt-in `harness.verify.commands` after edit/write/apply_patch |
| M6 Observability | shipped | opt-in `harness.trace` → `.bugbee/harness/trace.jsonl`; `bugbee doctor` |

### Config example

```jsonc
// bugbee.jsonc
{
  "harness": {
    "max_steps": 80,
    "memory": { "enabled": true, "dir": ".bugbee/memory" },
    "verify": {
      "enabled": true,
      "commands": ["bun test --timeout 30000"]
    },
    "trace": { "enabled": true }
  }
}
```

## Later

- Eval fixtures + CI harness scores  
- Pre-tool hooks (format/lint/secret scan packs)  
- Parallel review workers with merge report  

Keep UI minimal. Prefer correctness and control over features that shout.
