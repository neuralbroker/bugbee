---
name: security-hunt
description: Defensive vulnerability hunt workflow for Bugbee — scanners, evidence triage, status updates, SARIF/markdown export.
---

# Security hunt skill

Use when the user wants to audit, hunt, pen-test (authorized), review AppSec, or find vulnerabilities.

## Steps

1. **Scan** — `vuln_scan` with secrets enabled on the project root (or scoped path).
2. **Inventory** — `findings` action `list`, sort mentally by critical → high.
3. **Prove** — for each top finding, `Read`/`Grep` to confirm source→sink; `findings` `get` for stored evidence.
4. **Triage** — `set_status` to `confirmed` or `false_positive` with clear rationale.
5. **Fix** — minimal patches only for confirmed issues; add/adjust tests when possible.
6. **Export** — `security_report` markdown for humans, SARIF for CI.

## Hard rules

- Defense only; authorized targets only.
- Never print full secrets; urge rotation.
- No finding is “confirmed” without path:line evidence.
- Prefer scanners before speculative claims.
