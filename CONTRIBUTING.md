# Contributing to Bugbee

Thanks for helping build the AI coding IDE with defensive security built in.

## What we merge readily

- Bug fixes
- Additional LSPs / formatters / providers
- Security engine rules (high-signal, low false-positive)
- Secret detectors with redaction
- Documentation improvements
- Tests and eval fixtures
- Performance fixes

UI or core product features should be discussed with maintainers first (issue or design note).

If unsure, open an issue with labels such as `help wanted`, `good first issue`, `bug`, or `security-engine`.

## Security contributions

When adding rules or detectors:

1. Prefer **evidence-backed heuristics** over noisy keywords.
2. Always **redact** secret material in output.
3. Stay **defense-only** — no live exploit helpers, no weaponized payloads.
4. Add or extend fixtures under `fixtures/` and run:

   ```bash
   bun packages/bugbee/src/security/selftest.ts
   ```

5. Document CWE/tags when applicable (`packages/bugbee/src/security/rules.ts`).

## Developing Bugbee

**Requirements:** Bun 1.3+

```bash
bun install
bun run dev          # TUI against packages/bugbee by default
bun run dev .        # TUI against monorepo root
```

### Useful paths

| Path | Role |
|------|------|
| `packages/bugbee` | CLI entry, tools, security engine |
| `packages/bugbee/src/security` | Offline scanners, store, SARIF |
| `packages/bugbee/src/tool` | Agent tools |
| `packages/core` | Runtime, providers, plugins |
| `packages/app` | Shared web UI |
| `packages/desktop` | Desktop shell |
| `packages/plugin` | `@bugbee-ai/plugin` |

### Offline hunt while developing

```bash
bun run --cwd packages/bugbee --conditions=browser src/index.ts hunt ./fixtures/python-vuln
bun run --cwd packages/bugbee --conditions=browser src/index.ts findings --directory ./fixtures/python-vuln
```

### Building a local binary

```bash
./packages/bugbee/script/build.ts --single
./packages/bugbee/dist/bugbee-<platform>/bin/bugbee
```

### Conventions

See [AGENTS.md](./AGENTS.md) for coding style, commits, and branch naming.

- Conventional commits: `feat(scope): …`, `fix: …`, `docs: …`
- Prefer Bun APIs when reasonable
- Do not commit secrets, `.bugbee/findings.json`, or local reports

## Providers

Many providers work via configuration without code changes. Upstream model catalog contributions may still go to [models.dev](https://github.com/anomalyco/models.dev) where applicable.

## License

By contributing, you agree that your contributions are licensed under the MIT License (see [LICENSE](./LICENSE) and [NOTICE](./NOTICE)).
