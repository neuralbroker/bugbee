<p align="center">
  <strong>Bugbee</strong> — the open source AI coding agent
</p>

<p align="center">
  <a href="https://github.com/neuralbroker/bugbee"><img alt="GitHub" src="https://img.shields.io/badge/github-neuralbroker%2Fbugbee-blue?style=flat-square" /></a>
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

# Bugbee

Bugbee is an **AI coding agent** for the terminal, desktop, and IDE.

This repository is a **product fork** of [OpenCode](https://github.com/anomalyco/opencode) (MIT), rebranded and published as **Bugbee**.  
Not affiliated with the OpenCode team. Upstream credit belongs to OpenCode authors and contributors — see [NOTICE](./NOTICE) and [LICENSE](./LICENSE).

## Install (from source)

**Requirements:** [Bun](https://bun.sh) 1.3+

```bash
git clone https://github.com/neuralbroker/bugbee.git
cd bugbee
bun install --ignore-scripts

# CLI / TUI
bun run dev
# or
bun --conditions=browser packages/bugbee/src/index.ts
```

```bash
bugbee --help
bugbee agents list
bugbee providers
```

## Config

- Project: `bugbee.json` / `bugbee.jsonc` and `.bugbee/`
- Global: `~/.config/bugbee`
- Environment: `BUGBEE_*` (see `packages/core/src/flag/flag.ts`)

## Packages

| Package | Role |
|---------|------|
| `packages/bugbee` | CLI + agent entry |
| `packages/core` | Runtime, providers, plugins |
| `packages/app` / `desktop` / `tui` | UI shells |
| `packages/sdk` / `schema` / `plugin` | APIs & plugins |

## Develop

```bash
bun install --ignore-scripts
bun run dev
bun run --cwd packages/bugbee test
```

## Docs for contributors

- [AGENTS.md](./AGENTS.md) — repo conventions
- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [SECURITY.md](./SECURITY.md)

## License

MIT. Portions derived from OpenCode — see [NOTICE](./NOTICE).
