import { Effect, Schema } from "effect"
import { InstanceState } from "@/effect/instance-state"
import DESCRIPTION from "./secrets_scan.txt"
import * as Tool from "./tool"
import { hunt } from "../security/scanner"
import { mergeFindings } from "../security/store"

export const Parameters = Schema.Struct({
  path: Schema.optional(Schema.String).annotate({
    description: "Directory to scan. Defaults to the project root.",
  }),
})

export const SecretsScanTool = Tool.define(
  "secrets_scan",
  Effect.gen(function* () {
    return {
      description: DESCRIPTION,
      parameters: Parameters,
      execute: (params: { path?: string }, ctx: Tool.Context) =>
        Effect.gen(function* () {
          yield* ctx.ask({
            permission: "secrets_scan",
            patterns: [params.path ?? "*"],
            always: ["*"],
            metadata: { path: params.path },
          })

          const ins = yield* InstanceState.context
          const directory = params.path
            ? params.path.startsWith("/")
              ? params.path
              : `${ins.directory}/${params.path}`
            : ins.directory

          const report = yield* Effect.promise(() =>
            hunt({ directory, includeSecrets: true, includeRules: false }),
          )
          const store = yield* Effect.promise(() => mergeFindings(directory, report.findings))

          const open = report.findings.filter((f) => f.status === "draft")
          const lines = [
            `Secrets scan complete in ${directory}`,
            `Files scanned: ${report.filesScanned}`,
            `Secrets candidates: ${open.length}`,
            `Store: .bugbee/findings.json (${store.findings.length} total findings)`,
            "",
            "Top findings (redacted):",
          ]
          for (const f of open.slice(0, 20)) {
            lines.push(`- [${f.severity}] ${f.id} ${f.title} @ ${f.path}:${f.line}`)
            lines.push(`  ${f.snippet}`)
          }
          if (open.length === 0) lines.push("- None found by built-in detectors")
          lines.push("", "Defense-only: values are redacted. Rotate any real credentials immediately.")

          return {
            title: `Secrets scan: ${open.length} candidates`,
            metadata: {
              filesScanned: report.filesScanned,
              count: open.length,
              summary: report.summary,
            },
            output: lines.join("\n"),
          }
        }),
    }
  }),
)
