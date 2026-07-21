export * as CommandPlugin from "./command"

import { define } from "./internal"
import { Effect } from "effect"
import { Location } from "../location"
import PROMPT_INITIALIZE from "./command/initialize.txt"
import PROMPT_REVIEW from "./command/review.txt"
import PROMPT_HUNT from "./command/hunt.txt"
import PROMPT_FINDINGS from "./command/findings.txt"
import PROMPT_REPORT from "./command/report.txt"

export const Plugin = define({
  id: "command",
  effect: Effect.fn(function* (ctx) {
    const location = yield* Location.Service
    yield* ctx.command.transform((draft) => {
      draft.update("init", (command) => {
        command.template = PROMPT_INITIALIZE.replace("${path}", location.project.directory)
        command.description = "guided AGENTS.md setup"
      })
      draft.update("review", (command) => {
        command.template = PROMPT_REVIEW.replace("${path}", location.project.directory)
        command.description = "review changes [commit|branch|pr], defaults to uncommitted"
        command.subtask = true
      })
      draft.update("hunt", (command) => {
        command.template = PROMPT_HUNT.replace("${path}", location.project.directory)
        command.description = "defensive security hunt: scanners + evidence triage"
        command.agent = "hunt"
      })
      draft.update("findings", (command) => {
        command.template = PROMPT_FINDINGS.replace("${path}", location.project.directory)
        command.description = "list and triage open security findings"
      })
      draft.update("report", (command) => {
        command.template = PROMPT_REPORT.replace("${path}", location.project.directory)
        command.description = "export security report (markdown or SARIF)"
      })
    })
  }),
})
