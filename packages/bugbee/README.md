# Bugbee CLI entry point

This package contains the source entry point used by the repository's local Bugbee CLI and
development runner. It is not a standalone application template.

From the repository root, install dependencies with:

```bash
bun install --ignore-scripts
```

Run the local terminal agent with:

```bash
bun run dev
```

For the packaged CLI workflow, use [`packages/cli`](../cli) and the root
[`bin/bugbee`](../../bin/bugbee) launcher.
