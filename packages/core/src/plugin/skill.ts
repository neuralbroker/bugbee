/// <reference path="../markdown.d.ts" />

export * as SkillPlugin from "./skill"

import { define } from "./internal"
import { Effect } from "effect"
import { AbsolutePath } from "../schema"
import { SkillV2 } from "../skill"
import customizeBugbeeContent from "./skill/customize-bugbee.md" with { type: "text" }

export const CustomizeBugbeeContent = customizeBugbeeContent

export const Plugin = define({
  id: "skill",
  effect: Effect.fn(function* (ctx) {
    yield* ctx.skill.transform((draft) => {
      draft.source(
        SkillV2.EmbeddedSource.make({
          type: "embedded",
          skill: SkillV2.Info.make({
            name: "customize-bugbee",
            description:
              "Use ONLY when the user is editing or creating bugbee's own configuration: bugbee.json, bugbee.jsonc, files under .bugbee/, or files under ~/.config/bugbee/. Also use when creating or fixing bugbee agents, subagents, commands, skills, plugins, MCP servers, or permission rules. Do not use for the user's own application code, or for any project that is not configuring bugbee itself.",
            location: AbsolutePath.make("/builtin/customize-bugbee.md"),
            content: CustomizeBugbeeContent,
          }),
        }),
      )
    })
  }),
})
