import { Effect, Schema } from "effect"
import { InstanceState } from "@/effect/instance-state"
import DESCRIPTION from "./vuln_scan.txt"
import * as Tool from "./tool"
import { hunt } from "../security/scanner"
import { mergeFindings } from "../security/store"

export const Parameters = Schema.Struct({
  path: Schema.optional(Schema.String).annotate({
    description: "Directory to scan. Defaults to the project root.",
  }),
  include_secrets: Schema.optional(Schema.Boolean).annotate({
    description: "Also run secret detectors. Default true for full hunt.",
  }),
})

export const VulnScanTool = Tool.define(
  "vuln_scan",
  Effect.gen(function* () {
    return {
      description: DESCRIPTION,
      parameters: Parameters,
      execute: (params: { path?: string; include_secrets?: boolean }, ctx: Tool.Context) =>
        Effect.gen(function* () {
          yield* ctx.ask({
            permission: "vuln_scan",
            patterns: [params.path ?? "*"],
            always: ["*"],
            metadata: { path: params.path, include_secrets: params.include_secrets },
          })

          const ins = yield* InstanceState.context
          const directory = params.path
            ? params.path.startsWith("/")
              ? params.path
              : `${ins.directory}/${params.path}`
            : ins.directory

          const includeSecrets = params.include_secrets !== false
          const report = yield* Effect.promise(() =>
            hunt({ directory, includeSecrets, includeRules: true }),
          )
          const store = yield* Effect.promise(() => mergeFindings(directory, report.findings))

          const open = report.findings.filter((f) => f.status === "draft")
          const lines = [
            `Vulnerability hunt complete in ${directory}`,
            `Files scanned: ${report.filesScanned}`,
            `New draft findings this run: ${open.length}`,
            `Summary: critical=${report.summary.critical} high=${report.summary.high} medium=${report.summary.medium} low=${report.summary.low}`,
            `Store: .bugbee/findings.json (${store.findings.length} total)`,
            "",
            "Findings (draft — verify evidence before confirming):",
          ]
          for (const f of open.slice(0, 30)) {
            lines.push(`- [${f.severity}] ${f.id} ${f.title}`)
            lines.push(`  ${f.path}:${f.line} · ${f.ruleId}${f.cwe ? ` · ${f.cwe}` : ""}`)
            lines.push(`  ${f.message}`)
            if (f.snippet) lines.push(`  > ${f.snippet}`)
          }
          if (open.length === 0) lines.push("- No heuristic matches. Consider deeper agent review of auth/session/input paths.")
          lines.push(
            "",
            "Next steps: use findings tool to inspect evidence, confirm/fp/fixed, then security_report for SARIF/markdown.",
            "Defense-only: no live exploitation was performed.",
          )

          return {
            title: `Hunt: ${open.length} draft findings`,
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
