<p align="center">
  <strong>Bugbee</strong> — AI engineering with a paper trail
</p>

<p align="center">
  <a href="https://github.com/neuralbroker/bugbee"><img alt="GitHub" src="https://img.shields.io/badge/github-neuralbroker%2Fbugbee-blue?style=flat-square" /></a>
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

# Bugbee

Bugbee is an open-source AI engineering agent for the terminal, desktop, IDE, and API.
It keeps work durable, makes autonomy visible, and gives every change a reviewable trail.

Bugbee is built for developers who want an agent that can act across a real repository
without losing project context, tool history, permissions, or verification state.

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

For a guided readiness check inside a session, use:

```bash
bugbee
> /doctor
```

From a project directory, your first task can be as simple as:

```bash
bugbee
```

Bugbee will check the project, configuration, and providers before you begin. Use
`bugbee doctor` for the offline installation check, or `/init` inside a session to create
project-specific `AGENTS.md` guidance.

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

These controls make Bugbee useful for longer-running engineering work: sessions can resume,
project memory can persist, edits can be verified, and agent activity can be inspected.

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
| `packages/bugbee` / `packages/cli` | CLI and agent entry points |
| `packages/core` / `packages/server` | Durable runtime, providers, tools, and HTTP server |
| `packages/app` / `packages/desktop` / `packages/tui` | Web, desktop, and terminal clients |
| `packages/schema` / `packages/protocol` / `packages/client` | Shared contracts and generated clients |
| `packages/plugin` / `packages/codemode` | Extensions and confined tool orchestration |
| `packages/sdk` / `packages/sdk-next` | Embedded and programmatic Bugbee hosts |

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

## Product direction

Bugbee's core product is the durable engineering run: an objective, the agent actions it
took, the permissions it received, the files and commands it touched, and the verification
that followed. The terminal is the primary interface, but the same run can be consumed from
the desktop app, IDE integrations, shared web views, Slack, or the SDK.

The project favors controlled autonomy over opaque automation. See [CONTEXT.md](./CONTEXT.md)
for the session model and [docs/SUPERHARNESS.md](./docs/SUPERHARNESS.md) for agent-loop
controls.

## License

MIT. See [LICENSE](./LICENSE) for details.
