# Contributing to Bugbee

Thank you for helping build a memory-safe security agent.

## Principles

1. Read [VISION.md](./VISION.md) — PRs that contradict it need an explicit vision change.
2. Keep the core in **Rust**. No Node/Python runtime in the agent binary path.
3. Engine code must stay **offline** (no network in `bugbee-engine`).
4. Prefer tests + fixtures over screenshots of findings.

## Dev setup

```bash
rustup update stable
cargo build -p bugbee
cargo test --workspace
cargo run -p bugbee -- --root fixtures/python-vuln hunt
```

## Crate map

| Crate | Responsibility |
|-------|----------------|
| `bugbee-core` | Types, config, store, scoring, redaction |
| `bugbee-engine` | Deterministic scanners |
| `bugbee-llm` | BYOK model clients |
| `bugbee-agent` | Roles, tools, permissions, hunt orchestration |
| `bugbee-ui` | Ratatui workspace |
| `bugbee` | CLI binary |

## Style

- `cargo fmt` + `cargo clippy -p bugbee -- -D warnings` before sending a PR
- Small, reviewable commits
- New rules go under `rules/` with a clear id and CWE when known
