# Contributing to Bugbee

Bugbee welcomes bug reports, rule packs, vulnerable-but-safe fixtures, tests,
and documentation improvements.

## Before opening a pull request

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Keep changes focused. New detection rules should include a safe fixture that
demonstrates both the intended finding and a nearby non-finding where practical.
Never add real credentials, customer code, exploit payloads for live systems, or
harmful automation.

## Responsible disclosure

Do not open a public issue for a suspected security vulnerability. Follow the
[security policy](SECURITY.md) instead.
