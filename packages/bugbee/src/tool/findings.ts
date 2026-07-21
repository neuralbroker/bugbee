import { Effect, Schema } from "effect"
import { InstanceState } from "@/effect/instance-state"
import DESCRIPTION from "./findings.txt"
import * as Tool from "./tool"
import { loadFindings, updateFindingStatus } from "../security/store"
import type { FindingStatus, Severity } from "../security/types"

const StatusSchema = Schema.Literals(["draft", "confirmed", "false_positive", "fixed"])
const SeveritySchema = Schema.Literals(["critical", "high", "medium", "low", "info"])

export const Parameters = Schema.Struct({
  action: Schema.Literals(["list", "get", "set_status"]).annotate({
    description: "list open findings, get one by id, or set_status",
  }),
  id: Schema.optional(Schema.String).annotate({ description: "Finding id (for get / set_status)" }),
  status: Schema.optional(StatusSchema).annotate({ description: "New status for set_status" }),
  severity: Schema.optional(SeveritySchema).annotate({ description: "Filter for list" }),
  limit: Schema.optional(Schema.Number).annotate({ description: "Max rows for list (default 50)" }),
})

export const FindingsTool = Tool.define(
  "findings",
  Effect.gen(function* () {
    return {
      description: DESCRIPTION,
      parameters: Parameters,
      execute: (
        params: {
          action: "list" | "get" | "set_status"
          id?: string
          status?: FindingStatus
          severity?: Severity
          limit?: number
        },
        ctx: Tool.Context,
      ) =>
        Effect.gen(function* () {
          yield* ctx.ask({
            permission: "findings",
            patterns: ["*"],
            always: ["*"],
            metadata: { action: params.action, id: params.id },
          })

          const ins = yield* InstanceState.context
          const directory = ins.directory

          if (params.action === "list") {
            const store = yield* Effect.promise(() => loadFindings(directory))
            let items = store.findings.filter((f) => f.status !== "false_positive" && f.status !== "fixed")
            if (params.severity) items = items.filter((f) => f.severity === params.severity)
            const limit = params.limit ?? 50
            const slice = items.slice(0, limit)
            const lines = [
              `Open findings: ${items.length} (showing ${slice.length})`,
              `Updated: ${store.updatedAt}`,
              "",
            ]
            for (const f of slice) {
              lines.push(`[${f.severity}] ${f.id} · ${f.title} · ${f.path}:${f.line} · status=${f.status}`)
            }
            if (slice.length === 0) lines.push("No open findings. Run vuln_scan or secrets_scan first.")
            return {
              title: `Findings: ${items.length} open`,
              metadata: { count: items.length },
              output: lines.join("\n"),
            }
          }

          if (params.action === "get") {
            if (!params.id) throw new Error("id is required for get")
            const store = yield* Effect.promise(() => loadFindings(directory))
            const f = store.findings.find((x) => x.id === params.id)
            if (!f) {
              return {
                title: "Finding not found",
                metadata: {},
                output: `No finding with id ${params.id}`,
              }
            }
            const lines = [
              `# ${f.title}`,
              `id: ${f.id}`,
              `rule: ${f.ruleId}`,
              `severity: ${f.severity}`,
              `status: ${f.status}`,
              `location: ${f.path}:${f.line}`,
              f.cwe ? `cwe: ${f.cwe}` : "",
              `tags: ${f.tags.join(", ")}`,
              `source: ${f.source}`,
              "",
              f.message,
              "",
              "Snippet:",
              f.snippet || "(empty)",
              "",
              "Evidence:",
              ...f.evidence.map((e) => `- ${e}`),
            ].filter(Boolean)
            return {
              title: f.title,
              metadata: { id: f.id, severity: f.severity, status: f.status },
              output: lines.join("\n"),
            }
          }

          // set_status
          if (!params.id) throw new Error("id is required for set_status")
          if (!params.status) throw new Error("status is required for set_status")
          const updated = yield* Effect.promise(() =>
            updateFindingStatus(directory, params.id!, params.status!),
          )
          if (!updated) {
            return {
              title: "Finding not found",
              metadata: {},
              output: `No finding with id ${params.id}`,
            }
          }
          return {
            title: `Status → ${updated.status}`,
            metadata: { id: updated.id, status: updated.status },
            output: `Updated ${updated.id} (${updated.title}) status to ${updated.status}`,
          }
        }),
    }
  }),
)
