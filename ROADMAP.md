# Bugbee Roadmap

Status legend: **shipped** · **next** · **later**

## Shipped (foundation)

- [x] OpenCode monorepo forked and rebranded to Bugbee (CLI, packages, env, config dirs)
- [x] Dual-mandate default `build` agent (coding + defensive security)
- [x] `hunt` primary agent + `security-review` subagent
- [x] Tools: `secrets_scan`, `vuln_scan`, `findings`, `security_report`
- [x] Offline CLI: `bugbee hunt`, `bugbee findings`
- [x] Findings store (`.bugbee/findings.json`) with draft / confirmed / FP / fixed
- [x] SARIF 2.1.0 + markdown export
- [x] Slash commands: `/hunt`, `/findings`, `/report`
- [x] Defense-only constitution in system prompts
- [x] Fixture: `fixtures/python-vuln` + security selftest

## Next

- [ ] Deeper language packs (JS/TS, Python, Go, Java, PHP rule depth)
- [ ] Dependency / SCA reachability hints (lockfile-aware)
- [ ] VS Code / editor security panel (findings list + jump-to-line)
- [ ] CI GitHub Action: `bugbee hunt --format sarif` on PRs
- [ ] Eval harness: precision/recall gates on golden fixtures in CI
- [ ] Policy packs as data (enterprise YAML rule sets)
- [ ] Publish install path (`install` script, npm/binary) under Bugbee identity

## Later

- [ ] Attack Knowledge Graph (kill-chain linking of findings)
- [ ] Language-aware taint / dataflow (tree-sitter or engine adapters)
- [ ] Enterprise SSO / policy-gated cloud console (optional)
- [ ] Desktop security workspace polish
- [ ] Multi-repo / monorepo-scoped continuous hunt

## Non-goals (hard)

- Live exploitation modules against third-party systems
- Weaponized payload generation for unauthorized targets
- Replacing professional human AppSec review for high-assurance systems

See [VISION.md](./VISION.md) for the long-term north star.
