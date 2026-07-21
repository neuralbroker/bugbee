import fs from "fs/promises"
import path from "path"
import { Effect, Schema } from "effect"
import { InstanceState } from "@/effect/instance-state"
import DESCRIPTION from "./security_report.txt"
import * as Tool from "./tool"
import { loadFindings } from "../security/store"
import { toMarkdownReport, toSarif } from "../security/sarif"

export const Parameters = Schema.Struct({
  format: Schema.optional(Schema.Literals(["markdown", "sarif"])).annotate({
    description: "Report format. Default markdown.",
  }),
  path: Schema.optional(Schema.String).annotate({
    description: "Optional output path relative to project root.",
  }),
})

export const SecurityReportTool = Tool.define(
  "security_report",
  Effect.gen(function* () {
    return {
      description: DESCRIPTION,
      parameters: Parameters,
      execute: (params: { format?: "markdown" | "sarif"; path?: string }, ctx: Tool.Context) =>
        Effect.gen(function* () {
          yield* ctx.ask({
            permission: "security_report",
            patterns: [params.path ?? "*"],
            always: ["*"],
            metadata: { format: params.format, path: params.path },
          })

          const ins = yield* InstanceState.context
          const directory = ins.directory
          const store = yield* Effect.promise(() => loadFindings(directory))
          const format = params.format ?? "markdown"

          let body: string
          let defaultName: string
          if (format === "sarif") {
            body = JSON.stringify(toSarif(store.findings), null, 2) + "\n"
            defaultName = "bugbee-results.sarif.json"
          } else {
            body = toMarkdownReport(store.findings)
            defaultName = "bugbee-report.md"
          }

          let written: string | undefined
          if (params.path || format === "sarif") {
            const outRel = params.path ?? defaultName
            const outAbs = path.isAbsolute(outRel) ? outRel : path.join(directory, outRel)
            yield* Effect.promise(async () => {
              await fs.mkdir(path.dirname(outAbs), { recursive: true })
              await fs.writeFile(outAbs, body, "utf8")
            })
            written = outAbs
          }

          const open = store.findings.filter((f) => f.status !== "false_positive" && f.status !== "fixed")
          const header = [
            `Report format: ${format}`,
            `Open findings: ${open.length}`,
            written ? `Written: ${written}` : "Not written to disk (pass path to save)",
            "",
          ].join("\n")

          // Cap inline output for large reports
          const max = 12000
          const inline = body.length > max ? body.slice(0, max) + "\n\n… truncated; open the written file for full report." : body

          return {
            title: `Security report (${format})`,
            metadata: { format, count: open.length, path: written },
            output: header + inline,
          }
        }),
    }
  }),
)
