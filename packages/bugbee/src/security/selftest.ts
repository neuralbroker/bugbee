/**
 * Standalone security engine self-test (no monorepo deps beyond Bun/Node).
 * Run: bun packages/bugbee/src/security/selftest.ts
 */
import fs from "fs/promises"
import os from "os"
import path from "path"
import { hunt } from "./scanner"
import { mergeFindings, loadFindings, updateFindingStatus } from "./store"
import { toSarif, toMarkdownReport } from "./sarif"

async function main() {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), "bugbee-selftest-"))
  const vuln = path.join(dir, "app.py")
  await fs.writeFile(
    vuln,
    [
      "import os, pickle, hashlib",
      "password = 'SuperSecret123'",
      "def handle(req):",
      "    eval(req.body)",
      "    os.system(req.cmd)",
      "    return hashlib.md5(req.data).hexdigest()",
      "",
    ].join("\n"),
    "utf8",
  )
  await fs.writeFile(path.join(dir, "leaked.env"), 'OPENAI_KEY=sk-abcdefghijklmnopqrstuvwxyz012345\n', "utf8")

  const report = await hunt({ directory: dir, includeSecrets: true, includeRules: true })
  if (report.filesScanned < 2) throw new Error(`expected files scanned, got ${report.filesScanned}`)
  if (report.findings.length < 3) throw new Error(`expected multiple findings, got ${report.findings.length}`)

  const hasEval = report.findings.some((f) => f.ruleId.includes("eval") || f.title.toLowerCase().includes("eval"))
  if (!hasEval) throw new Error("expected eval finding")

  const hasSecret = report.findings.some((f) => f.source === "secrets" || f.tags.includes("secrets"))
  if (!hasSecret) throw new Error("expected secret finding")

  // Ensure redaction
  const secretFinding = report.findings.find((f) => f.source === "secrets")
  if (secretFinding && secretFinding.snippet.includes("sk-abcdefghijklmnopqrstuvwxyz012345")) {
    throw new Error("secret was not redacted in snippet")
  }

  const store = await mergeFindings(dir, report.findings)
  if (store.findings.length < 3) throw new Error("store merge failed")

  const first = store.findings[0]!
  const updated = await updateFindingStatus(dir, first.id, "confirmed")
  if (!updated || updated.status !== "confirmed") throw new Error("status update failed")

  const reloaded = await loadFindings(dir)
  if (!reloaded.findings.some((f) => f.id === first.id && f.status === "confirmed")) {
    throw new Error("status not persisted")
  }

  const sarif = toSarif(reloaded.findings)
  if (sarif.version !== "2.1.0" || !sarif.runs[0]?.results?.length) throw new Error("sarif invalid")

  const md = toMarkdownReport(reloaded.findings)
  if (!md.includes("Bugbee Security Report")) throw new Error("markdown report missing title")

  console.log("security selftest OK")
  console.log(
    JSON.stringify(
      {
        filesScanned: report.filesScanned,
        findings: report.findings.length,
        summary: report.summary,
        sample: report.findings.slice(0, 5).map((f) => ({
          id: f.id,
          rule: f.ruleId,
          sev: f.severity,
          at: `${f.path}:${f.line}`,
        })),
      },
      null,
      2,
    ),
  )

  await fs.rm(dir, { recursive: true, force: true })
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
