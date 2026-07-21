# Agent & contributor guide (Bugbee)

This repository is **Bugbee** — an AI coding agent with first-class defensive security.

## Product facts for agents

- Binary / CLI name: `bugbee`
- Config: `bugbee.json` / `bugbee.jsonc`, directory `.bugbee/`
- Env prefix: `BUGBEE_*`
- Package scope: `@bugbee-ai/*`
- Offline hunt: `bugbee hunt` (no LLM)
- Security tools: `secrets_scan`, `vuln_scan`, `findings`, `security_report`
- Agents: `build` (default), `plan`, `hunt`, `security-review`, `explore`, `general`
- Defense only — never add live exploitation capabilities

## Regenerating SDKs / API

- To regenerate the legacy JavaScript SDK, run `./packages/sdk/js/script/build.ts`.
- After changing the public Protocol or Server `HttpApi`, run `bun run generate` from `packages/client`. Do not edit `src/generated` or `src/generated-effect` directly.
- Keep runtime dependencies directed from Schema to Core and Protocol, then from Core and Protocol to Server. Client runtime code may depend on Schema and Protocol but never Core or Server; `sdk-next` composes Client, Core, and Server.

## Branches

Default working branch for this product fork is typically `main` (product remote: `neuralbroker/bugbee`). Upstream OpenCode used `dev`.

Use a short branch name of at most three words, separated by hyphens. Do not use slashes or type prefixes such as `feat/` or `fix/`.

Examples: `session-recovery`, `hunt-rules-pack`, `sarif-export`.

## Commits and PR titles

Use conventional commit-style messages and PR titles: `type(scope): summary`.

Valid types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`.  
Scopes (optional): `core`, `bugbee`, `security`, `tui`, `app`, `desktop`, `sdk`, `plugin`.

Examples: `feat(security): add pickle deserialization rule`, `docs: refresh product README`.

## Style guide

### General principles

- Keep things in one function unless composable or reusable
- Do not extract single-use helpers preemptively
- Avoid `try`/`catch` where possible
- Avoid using the `any` type
- Use Bun APIs when possible, like `Bun.file()`
- Rely on type inference when possible
- Prefer functional array methods over for loops when clarity is equal
- In `src/config`, follow the existing self-export pattern when adding a new config module
- In Effect generators, bind services to named variables before calling methods

### Destructuring

Avoid unnecessary destructuring. Use dot notation to preserve context.

### Imports

- Never alias imports
- Never use star imports for local modules when a named namespace export exists
- Prefer dynamic imports for heavy, branch-specific modules

### Variables

Prefer `const` over `let`. Use ternaries or early returns instead of reassignment.

### Security engine

- Rules live in `packages/bugbee/src/security/rules.ts`
- Secrets patterns in `packages/bugbee/src/security/secrets.ts`
- Always redact secret matches
- Run `bun packages/bugbee/src/security/selftest.ts` after changes
- Do not introduce network-calling exploit tooling

## Docs map

| File | Use |
|------|-----|
| `README.md` | User-facing product docs |
| `VISION.md` | Strategy and constitution |
| `ROADMAP.md` | Shipped / next / later |
| `SECURITY.md` | Vuln reporting + posture |
| `CONTRIBUTING.md` | Human contributor onboarding |
