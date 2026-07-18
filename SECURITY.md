# Security Policy

## Defensive use only

Bugbee is designed for **authorized, defensive** assessment of codebases you own
or have explicit permission to test. It does not include live exploitation
tooling against third-party systems.

## Reporting vulnerabilities in Bugbee

If you discover a security issue **in Bugbee itself** (the agent, store,
redaction, or install path), please report it privately:

- Email: security@neuralbroker.dev (or open a private security advisory on GitHub)
- Do not file public issues for unfixed critical vulnerabilities

Please include:

1. Affected version / commit
2. Reproduction steps
3. Impact assessment
4. Any suggested fix

We aim to acknowledge reports within 72 hours.

## Secrets and keys

- Prefer environment variables or OS keychain via `bugbee connect` (when available)
- Never commit API keys or `.bugbee/` state containing credentials
- Outbound LLM prompts should pass through redaction before leaving the host

## Supply chain

- Prefer `cargo install --locked` or signed release binaries
- Review third-party crate updates carefully; core stays Rust / memory-safe
