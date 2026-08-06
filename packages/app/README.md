# @bugbee-ai/app

The browser application for Bugbee. It connects to a Bugbee server and provides the visual
workspace for sessions, diffs, permissions, terminals, settings, and project navigation.

The app is also consumed by the Electron desktop package, so shared UI and session behavior
belongs here while desktop-only concerns stay in `packages/desktop`.

## Development

Run from this package directory:

```bash
bun run dev
```

The Vite app uses its configured local port. A Bugbee backend is expected at `localhost:4096`
unless overridden by the app's server settings.

## Build and test

```bash
bun run build
bun run test
```

Browser tests use Playwright:

```bash
bunx playwright install chromium
bun run test:e2e:local
bun run test:e2e:local -- --grep settings
```

The backend and frontend ports can be configured with `PLAYWRIGHT_SERVER_HOST`,
`PLAYWRIGHT_SERVER_PORT`, `PLAYWRIGHT_PORT`, and `PLAYWRIGHT_BASE_URL`.

Type checking is run from the repository root with `bun typecheck`.
