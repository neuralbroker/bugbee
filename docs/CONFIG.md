# Bugbee configuration (Superharness keys)

All keys live under `harness` in `bugbee.jsonc` (legacy alias:
`experimental.harness`; top-level `harness` wins on conflict). Verified against
`packages/bugbee/src/harness/config.ts` — this doc mirrors `readHarness()`.

```jsonc
// bugbee.jsonc
{
  // Default max agent steps when agent.steps is unset. Default 80.
  // "max_steps": 80,
  "harness": {
    "max_steps": 80,
    "memory": {
      "enabled": true,          // load .bugbee/memory/*.md as instructions
      "dir": ".bugbee/memory"   // relative to project root
    },
    "verify": {
      "enabled": false,         // run commands after mutating tools
      "commands": ["bun test"], // cwd = project root, sequential
      "after_tools": ["edit", "write", "apply_patch"],
      "max_output_chars": 4000, // injected into tool output per run
      "timeout_ms": 120000      // SIGTERM, then SIGKILL after 2s
    },
    "trace": {
      "enabled": false          // append JSONL to .bugbee/harness/trace.jsonl
    }
  }
}
```

## Defaults rationale

| Key | Default | Why |
|---|---|---|
| `verify.enabled` | `false` | Verification costs seconds per mutation; projects opt in once commands are green and fast |
| `memory.enabled` | `true` | Notes are cheap and only load when files exist (≤20 files, 8KB each) |
| `trace.enabled` | `false` | Disk grows per tool call; enable when debugging or building evals |
| `max_steps` | `80` | Doom-loop ceiling; plan/review modes use less |

## Check health

```bash
./bin/bugbee doctor   # offline install health (no network)
```

## Files the harness owns

- `.bugbee/memory/*.md` — durable notes (you write these)
- `.bugbee/harness/trace.jsonl` — tool-step trace (machine-written, gitignore it)
