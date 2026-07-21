import { run as runTui, type TuiInput } from "@bugbee-ai/tui"
import { Global } from "@bugbee-ai/core/global"
import { AppNodeBuilder } from "@bugbee-ai/core/effect/app-node-builder"
import { Effect } from "effect"

export function run(input: TuiInput) {
  return runTui(input).pipe(Effect.provide(AppNodeBuilder.build(Global.node)))
}
