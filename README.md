<p align="center">
  <strong>Bugbee</strong> — the open source AI coding agent
</p>

<p align="center">
  <a href="https://github.com/neuralbroker/bugbee"><img alt="GitHub" src="https://img.shields.io/badge/github-neuralbroker%2Fbugbee-blue?style=flat-square" /></a>
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

# Bugbee

Bugbee is an **AI coding agent** for the terminal, desktop, and IDE.

Bugbee is published under the MIT License. See [LICENSE](./LICENSE) for details.

## Install

**Install script (recommended):**

```bash
curl -fsSL https://github.com/neuralbroker/bugbee/install | bash
```

**Package managers:**

```bash
npm install -g @neuralbroker/bugbee
# or: bun install -g @neuralbroker/bugbee
# or: pnpm install -g @neuralbroker/bugbee
# or: brew install neuralbroker/tap/bugbee
```

> **Note:** The bare npm name `bugbee` is blocked by npm’s typo-squatting rules
> (too similar to an existing package). The CLI package is **`@neuralbroker/bugbee`**;
> the installed command is still **`bugbee`**.

Then run:

```bash
bugbee
bugbee doctor
```

### From source

**Requirements:** [Bun](https://bun.sh) 1.3+

```bash
git clone https://github.com/neuralbroker/bugbee.git
cd bugbee
bun install --ignore-scripts

# CLI / TUI (must use packages/bugbee cwd for OpenTUI JSX preload)
bun run dev
# or
./bin/bugbee
# or
bun run --cwd packages/bugbee --conditions=browser src/index.ts
```

```bash
./bin/bugbee doctor
./bin/bugbee --help
./bin/bugbee agent list
./bin/bugbee providers
```

## Themes

Default theme is `bugbee`. Transparent terminal background:

```jsonc
// tui.json / ~/.config/bugbee/tui.json
{ "theme": "transparent" }
```

## Superharness

Optional agent-loop controls (see [docs/SUPERHARNESS.md](./docs/SUPERHARNESS.md)):

```jsonc
// bugbee.jsonc
{
  "harness": {
    "max_steps": 80,
    "memory": { "enabled": true },
    "verify": {
      "enabled": true,
      "commands": ["bun test"]
    },
    "trace": { "enabled": true }
  }
}
```

- **memory** — loads `.bugbee/memory/*.md` into context  
- **verify** — runs commands after edit/write/apply_patch (opt-in)  
- **trace** — appends tool steps to `.bugbee/harness/trace.jsonl`  
- **review** agent — read-only subagent for adversarial review  

```bash
./bin/bugbee doctor
./bin/bugbee agent list   # includes review
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

## Maintainers: cut a public release

Global users install **your** builds from GitHub Releases + npm (`bugbee`).

### One-time setup

1. Create an [npm](https://www.npmjs.com) account (2FA recommended).
2. Create an npm **Automation** token (Access Tokens).
3. Add it to this repo:

```bash
gh secret set NPM_TOKEN -R neuralbroker/bugbee
```

4. Ensure GitHub Actions are enabled for the repo.

### Publish CLI (npm + curl install assets)

```bash
gh workflow run release-cli.yml -R neuralbroker/bugbee -f version=1.0.0
```

This builds platform binaries, attaches them to `v1.0.0`, and runs `npm publish` for `@neuralbroker/bugbee` plus platform packages (`bugbee-linux-x64`, …).

After it finishes:

```bash
# anyone on the internet
curl -fsSL https://github.com/neuralbroker/bugbee/install | bash
npm install -g @neuralbroker/bugbee
bugbee --version
bugbee doctor
```

### Notes

- npm meta package is **`@neuralbroker/bugbee`** (bare `bugbee` is blocked by npm).
- Homebrew / Scoop / Chocolatey / AUR are optional later; they should point at the same GitHub Release assets.
- The full `publish.yml` workflow is for desktop + signing + extras and needs more secrets/runners.

## Docs for contributors

- [AGENTS.md](./AGENTS.md) — repo conventions
- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [SECURITY.md](./SECURITY.md)

## License

MIT. See [LICENSE](./LICENSE) for details.
