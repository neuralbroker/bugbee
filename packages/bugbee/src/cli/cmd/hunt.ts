import path from "path"
import fs from "fs/promises"
import { cmd } from "./cmd"
import { UI } from "../ui"
import { hunt } from "../../security/scanner"
import { mergeFindings, loadFindings, updateFindingStatus } from "../../security/store"
import { toMarkdownReport, toSarif } from "../../security/sarif"

/**
 * Offline, deterministic security hunt — no LLM required.
 * Full agentic triage remains available via TUI `/hunt` or `bugbee run --agent hunt`.
 */
export const HuntCommand = cmd({
  command: "hunt [directory]",
  describe: "run defensive scanners (secrets + rules) offline — no LLM required",
  builder: (yargs) =>
    yargs
      .positional("directory", {
        type: "string",
        describe: "directory to scan (default: cwd)",
      })
      .option("no-secrets", {
        type: "boolean",
        default: false,
        describe: "skip secret detectors",
      })
      .option("no-rules", {
        type: "boolean",
        default: false,
        describe: "skip vulnerability rule pack",
      })
      .option("format", {
        type: "string",
        choices: ["text", "json", "markdown", "sarif"] as const,
        default: "text",
        describe: "output format",
      })
      .option("output", {
        type: "string",
        alias: "o",
        describe: "write report to file",
      })
      .option("json", {
        type: "boolean",
        default: false,
        describe: "alias for --format json",
      }),
  handler: async (args) => {
    const directory = path.resolve((args.directory as string | undefined) || process.cwd())
    const includeSecrets = !args["no-secrets"]
    const includeRules = !args["no-rules"]
    const format = (args.json ? "json" : (args.format as string)) || "text"

    UI.println(UI.Style.TEXT_NORMAL_BOLD + "Bugbee hunt" + UI.Style.TEXT_NORMAL)
    UI.println(`directory: ${directory}`)
    UI.println(`engines: ${[includeSecrets && "secrets", includeRules && "rules"].filter(Boolean).join(" + ") || "none"}`)

    const report = await hunt({ directory, includeSecrets, includeRules })
    const store = await mergeFindings(directory, report.findings)

    const open = store.findings.filter((f) => f.status !== "false_positive" && f.status !== "fixed")

    if (format === "json") {
      const body = JSON.stringify({ report, store: { count: store.findings.length, open: open.length } }, null, 2)
      if (args.output) await fs.writeFile(args.output as string, body + "\n")
      else process.stdout.write(body + "\n")
      return
    }

    if (format === "markdown") {
      const body = toMarkdownReport(store.findings)
      const out = (args.output as string | undefined) || path.join(directory, "bugbee-report.md")
      await fs.writeFile(out, body)
      UI.println(`wrote ${out}`)
      UI.println(`open findings: ${open.length}`)
      return
    }

    if (format === "sarif") {
      const body = JSON.stringify(toSarif(store.findings), null, 2) + "\n"
      const out = (args.output as string | undefined) || path.join(directory, "bugbee-results.sarif.json")
      await fs.writeFile(out, body)
      UI.println(`wrote ${out}`)
      UI.println(`open findings: ${open.length}`)
      return
    }

    // text
    UI.println(`files scanned: ${report.filesScanned}`)
    UI.println(
      `summary: critical=${report.summary.critical} high=${report.summary.high} medium=${report.summary.medium} low=${report.summary.low}`,
    )
    UI.println(`store: ${path.join(directory, ".bugbee", "findings.json")} (${store.findings.length} total)`)
    UI.println("")
    for (const f of open.slice(0, 40)) {
      UI.println(`[${f.severity}] ${f.id} ${f.title}`)
      UI.println(`  ${f.path}:${f.line} · ${f.ruleId}${f.cwe ? ` · ${f.cwe}` : ""}`)
      if (f.snippet) UI.println(`  > ${f.snippet.slice(0, 160)}`)
    }
    if (open.length > 40) UI.println(`… and ${open.length - 40} more`)
    if (open.length === 0) UI.println("No open findings.")
    UI.println("")
    UI.println("Next: bugbee run --agent hunt \"triage top findings\"  or  bugbee findings")
    UI.println("Defense-only. No live exploitation was performed.")

    if (args.output) {
      const body = open
        .map((f) => `[${f.severity}] ${f.id} ${f.title} @ ${f.path}:${f.line}\n  ${f.message}`)
        .join("\n")
      await fs.writeFile(args.output as string, body + "\n")
      UI.println(`wrote ${args.output}`)
    }
  },
})

export const FindingsCommand = cmd({
  command: "findings",
  describe: "list or update security findings in .bugbee/findings.json",
  builder: (yargs) =>
    yargs
      .option("directory", {
        type: "string",
        describe: "project directory (default: cwd)",
      })
      .option("severity", {
        type: "string",
        choices: ["critical", "high", "medium", "low", "info"] as const,
      })
      .option("status", {
        type: "string",
        describe: "set status for --id (draft|confirmed|false_positive|fixed)",
      })
      .option("id", {
        type: "string",
        describe: "finding id for status update or detail",
      }),
  handler: async (args) => {
    const directory = path.resolve((args.directory as string | undefined) || process.cwd())
    if (args.id && args.status) {
      const updated = await updateFindingStatus(directory, args.id as string, args.status as any)
      if (!updated) {
        UI.error(`finding not found: ${args.id}`)
        process.exitCode = 1
        return
      }
      UI.println(`updated ${updated.id} → ${updated.status}`)
      return
    }
    const store = await loadFindings(directory)
    let items = store.findings
    if (args.id) {
      items = items.filter((f) => f.id === args.id)
    } else {
      items = items.filter((f) => f.status !== "false_positive" && f.status !== "fixed")
      if (args.severity) items = items.filter((f) => f.severity === args.severity)
    }
    if (items.length === 0) {
      UI.println("No findings. Run: bugbee hunt")
      return
    }
    for (const f of items) {
      UI.println(`[${f.severity}] ${f.id} ${f.title} · ${f.path}:${f.line} · ${f.status}`)
      if (args.id) {
        UI.println(f.message)
        for (const e of f.evidence) UI.println(`  - ${e}`)
      }
    }
    UI.println(`${items.length} finding(s)`)
  },
})
