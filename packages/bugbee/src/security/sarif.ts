import type { Finding, Severity } from "./types"

/** SARIF 2.1.0 export for CI / GRC integrations */
export function toSarif(findings: Finding[], toolName = "Bugbee") {
  const rulesMap = new Map<string, { id: string; name: string; shortDescription: { text: string }; helpUri?: string; properties?: Record<string, string> }>()
  for (const f of findings) {
    if (!rulesMap.has(f.ruleId)) {
      rulesMap.set(f.ruleId, {
        id: f.ruleId,
        name: f.title,
        shortDescription: { text: f.message },
        properties: f.cwe ? { cwe: f.cwe } : undefined,
      })
    }
  }

  return {
    $schema: "https://json.schemastore.org/sarif-2.1.0.json",
    version: "2.1.0",
    runs: [
      {
        tool: {
          driver: {
            name: toolName,
            informationUri: "https://github.com/neuralbroker/bugbee",
            rules: Array.from(rulesMap.values()),
          },
        },
        results: findings
          .filter((f) => f.status !== "false_positive" && f.status !== "fixed")
          .map((f) => ({
            ruleId: f.ruleId,
            level: sarifLevel(f.severity),
            message: { text: f.message },
            locations: [
              {
                physicalLocation: {
                  artifactLocation: { uri: f.path },
                  region: {
                    startLine: f.line,
                    snippet: { text: f.snippet },
                  },
                },
              },
            ],
            properties: {
              bugbeeId: f.id,
              status: f.status,
              tags: f.tags,
              evidence: f.evidence,
              cwe: f.cwe,
            },
          })),
      },
    ],
  }
}

function sarifLevel(severity: Severity): "error" | "warning" | "note" {
  if (severity === "critical" || severity === "high") return "error"
  if (severity === "medium") return "warning"
  return "note"
}

export function toMarkdownReport(findings: Finding[], title = "Bugbee Security Report"): string {
  const active = findings.filter((f) => f.status !== "false_positive" && f.status !== "fixed")
  const lines: string[] = [
    `# ${title}`,
    "",
    `Generated: ${new Date().toISOString()}`,
    "",
    `Total open findings: **${active.length}**`,
    "",
    "| Severity | Count |",
    "|----------|------:|",
  ]
  for (const sev of ["critical", "high", "medium", "low", "info"] as const) {
    const n = active.filter((f) => f.severity === sev).length
    if (n) lines.push(`| ${sev} | ${n} |`)
  }
  lines.push("", "## Findings", "")
  for (const f of active) {
    lines.push(`### ${f.severity.toUpperCase()}: ${f.title}`)
    lines.push("")
    lines.push(`- **ID:** \`${f.id}\``)
    lines.push(`- **Rule:** \`${f.ruleId}\``)
    lines.push(`- **Location:** \`${f.path}:${f.line}\``)
    if (f.cwe) lines.push(`- **CWE:** ${f.cwe}`)
    lines.push(`- **Status:** ${f.status}`)
    lines.push(`- **Message:** ${f.message}`)
    if (f.snippet) {
      lines.push("", "```", f.snippet, "```")
    }
    if (f.evidence.length) {
      lines.push("", "**Evidence:**")
      for (const e of f.evidence) lines.push(`- ${e}`)
    }
    lines.push("")
  }
  lines.push("---", "", "_Defense-only report. Verify every finding before acting. No live exploitation was performed._", "")
  return lines.join("\n")
}
