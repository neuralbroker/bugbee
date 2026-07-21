# Security Policy

## Product posture

Bugbee is a **defense-only** AI coding and security engineering tool.

| Principle | Practice |
|-----------|----------|
| Authorized use | Intended for repos/systems you own or are authorized to assess |
| Evidence first | Scanners produce draft findings with path/line/snippet |
| Secret safety | Detectors redact matched values in tool output |
| No weapons | No modules for live exploitation of third-party systems |

Hard laws are also encoded in agent system prompts and the `hunt` / `security-review` agent constitutions.

## Reporting a vulnerability in Bugbee itself

If you find a security issue in Bugbee (CLI, agent runtime, desktop, server, or scanners):

1. **Do not** open a public GitHub issue with exploit details.
2. Prefer a [private security advisory](https://github.com/neuralbroker/bugbee/security/advisories/new) on the repository, or contact maintainers via the GitHub organization.
3. Include: impact, affected versions/commits, reproduction steps, and any suggested fix.

We aim to acknowledge reports within **7 days** and ship fixes as quickly as practical.

## Using Bugbee safely

- Run hunts only on authorized targets.
- Treat automated findings as **draft** until verified.
- Rotate any real secrets discovered; do not paste them into tickets or chat logs.
- Review patches before applying to production.

## Authorized use only

Users are responsible for ensuring their use of Bugbee complies with law and organizational policy. Unauthorized scanning or access of systems is prohibited.

## Related

- [VISION.md](./VISION.md) — product constitution
- [NOTICE](./NOTICE) — upstream attribution
