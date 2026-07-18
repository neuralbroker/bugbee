# SuperHarness — Analysis & Integration Map

Bugbee’s **SuperHarness** synthesizes the best agent-runtime ideas from three public projects into a single Rust harness specialized for defensive security.

| Project | Repo | What we took |
|---------|------|----------------|
| **Pi** | [earendil-works/pi](https://github.com/earendil-works/pi) | Agent loop lifecycle, parallel tool batches, before/after tool hooks, steering + follow-up queues, fail tools on `length` truncation, context transform/compaction boundary |
| **OpenCode** | [anomalyco/opencode](https://github.com/anomalyco/opencode) | Permission allow/ask/deny, doom-loop detection, max steps, session compaction/prune, plan vs build roles, tool registry |
| **Claude Code** | [anthropics/claude-code](https://github.com/anthropics/claude-code) | Hook bus (PreToolUse/PostToolUse), Ralph outer loop + completion promise, catastrophic-command blocks, plugin-style security rules |

> Note: Claude Code’s public GitHub tree is primarily plugins/examples (not the full proprietary core). Patterns are taken from plugins (`hookify`, `ralph-wiggum`), changelog, and documented permission/hook behavior.

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │           SuperHarness              │
                    │  (loop_ · hooks · parallel · ralph) │
                    └──────────────┬──────────────────────┘
           ┌───────────────────────┼───────────────────────┐
           ▼                       ▼                       ▼
    ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
    │ Pi-style    │        │ OpenCode    │        │ Claude-style│
    │ turns/events│        │ budget/doom │        │ hooks/ralph │
    │ parallel    │        │ permissions │        │ completion  │
    │ steering    │        │ compaction  │        │ promise     │
    └─────────────┘        └─────────────┘        └─────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
            Bugbee tools                    NSAE / Swarm
         (hunt,grep,read,…)              (adjudicate, prove)
```

## Module map (`crates/bugbee-agent/src/superharness/`)

| Module | Responsibility |
|--------|----------------|
| `loop_.rs` | Main agent loop (Pi `runLoop` + OpenCode processor) |
| `hooks.rs` | Hook bus + defense defaults |
| `parallel.rs` | Sequential vs parallel-safe tool batches |
| `compaction.rs` | Context window prune without extra LLM |
| `ralph.rs` | Outer iteration until completion promise |
| `types.rs` | Unified `SuperEvent` stream |

## Lifecycle (one SuperHarness run)

```
agent_start
  └─ turn_start
       ├─ drain steering queue          (Pi)
       ├─ compact if over budget        (OpenCode/Pi)
       ├─ LLM call (+ tools)
       ├─ if tool_calls:
       │    ├─ doom-loop check          (OpenCode)
       │    ├─ fail if finish=length    (Pi)
       │    ├─ pre-tool hooks           (Claude/Pi)
       │    ├─ execute batch (// reads) (Pi)
       │    └─ post-tool hooks
       ├─ turn_end
       └─ should continue?
  └─ drain follow-up queue              (Pi)
agent_end
```

## CLI

```bash
# Single SuperHarness run (needs connected provider)
bugbee connect --provider ollama --model qwen2.5-coder --base-url http://127.0.0.1:11434/v1
bugbee super -v "Run hunt and summarize top risks with evidence"

# Ralph outer loop (max 3 iterations until BUGBEE_COMPLETE)
bugbee super --ralph 3 -v "Fully adjudicate all high findings"

# Still available:
bugbee swarm -v     # neuro-symbolic multi-agent pipeline (offline OK)
bugbee godmode -v   # uses SuperHarness under AgentRunner when LLM on
```

## Compatibility

- `AgentRunner` now **delegates** to SuperHarness (godmode LLM phases inherit all improvements).
- Swarm pipeline remains the offline/NSAE backbone; SuperHarness is the interactive tool-loop spine.

## Non-goals (not forked)

We do **not** vendor TypeScript runtimes from Pi/OpenCode/Claude Code.  
We re-implement the harness **contracts** in Rust for memory safety and air-gap binary delivery.
