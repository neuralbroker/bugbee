import { $ } from "bun"

await $`bun ./scripts/copy-icons.ts ${process.env.BUGBEE_CHANNEL ?? "dev"}`

await $`cd ../bugbee && bun script/build-node.ts`
