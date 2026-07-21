import { describe, expect, test } from "bun:test"
import fs from "fs/promises"
import os from "os"
import path from "path"
import { hunt } from "../../src/security/scanner"
import { mergeFindings, loadFindings, updateFindingStatus } from "../../src/security/store"
import { toSarif, toMarkdownReport } from "../../src/security/sarif"

describe("security.scanner", () => {
  test("detects eval, secrets, and redacts values", async () => {
    const dir = await fs.mkdtemp(path.join(os.tmpdir(), "bugbee-scan-"))
    await fs.writeFile(path.join(dir, "app.py"), "eval(user)\nos.system(cmd)\npassword = 'SuperSecret123'\n")
    await fs.writeFile(path.join(dir, "keys.env"), "OPENAI_KEY=sk-abcdefghijklmnopqrstuvwxyz012345\n")

    const report = await hunt({ directory: dir })
    expect(report.filesScanned).toBeGreaterThanOrEqual(2)
    expect(report.findings.length).toBeGreaterThanOrEqual(3)
    expect(report.findings.some((f) => f.ruleId.includes("eval"))).toBe(true)
    const secret = report.findings.find((f) => f.source === "secrets")
    expect(secret).toBeDefined()
    expect(secret!.snippet.includes("sk-abcdefghijklmnopqrstuvwxyz012345")).toBe(false)

    const store = await mergeFindings(dir, report.findings)
    expect(store.findings.length).toBeGreaterThanOrEqual(3)
    const first = store.findings[0]!
    await updateFindingStatus(dir, first.id, "confirmed")
    const reloaded = await loadFindings(dir)
    expect(reloaded.findings.find((f) => f.id === first.id)?.status).toBe("confirmed")

    const sarif = toSarif(reloaded.findings)
    expect(sarif.version).toBe("2.1.0")
    expect(sarif.runs[0]!.results!.length).toBeGreaterThan(0)
    expect(toMarkdownReport(reloaded.findings)).toContain("Bugbee Security Report")

    await fs.rm(dir, { recursive: true, force: true })
  })
})
