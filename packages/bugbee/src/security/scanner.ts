import fs from "fs/promises"
import path from "path"
import { createHash, randomUUID } from "crypto"
import { BUILTIN_RULES, pathMatchesRule } from "./rules"
import { SECRET_PATTERNS } from "./secrets"
import type { Finding, HuntReport, Rule, Severity } from "./types"

const SKIP_DIRS = new Set([
  "node_modules",
  ".git",
  ".hg",
  ".svn",
  "dist",
  "build",
  "target",
  "vendor",
  ".next",
  ".nuxt",
  "coverage",
  ".cache",
  ".turbo",
  ".bun",
  "__pycache__",
  ".venv",
  "venv",
  ".bugbee",
  "out",
  "bin",
  "obj",
])

const SKIP_EXT = new Set([
  ".png",
  ".jpg",
  ".jpeg",
  ".gif",
  ".webp",
  ".ico",
  ".woff",
  ".woff2",
  ".ttf",
  ".eot",
  ".mp4",
  ".webm",
  ".zip",
  ".gz",
  ".tgz",
  ".br",
  ".wasm",
  ".so",
  ".dylib",
  ".dll",
  ".exe",
  ".bin",
  ".pdf",
  ".lock",
  ".map",
  ".min.js",
  ".min.css",
])

const MAX_FILE_BYTES = 512 * 1024
const MAX_FILES = 5000

export interface ScanOptions {
  directory: string
  includeSecrets?: boolean
  includeRules?: boolean
  maxFiles?: number
  rules?: Rule[]
}

export async function hunt(options: ScanOptions): Promise<HuntReport> {
  const startedAt = new Date().toISOString()
  const directory = path.resolve(options.directory)
  const includeSecrets = options.includeSecrets !== false
  const includeRules = options.includeRules !== false
  const maxFiles = options.maxFiles ?? MAX_FILES
  const rules = options.rules ?? BUILTIN_RULES

  const files = await listSourceFiles(directory, maxFiles)
  const findings: Finding[] = []

  for (const file of files) {
    let content: string
    try {
      const buf = await fs.readFile(file)
      if (buf.byteLength > MAX_FILE_BYTES) continue
      if (buf.includes(0)) continue
      content = buf.toString("utf8")
    } catch {
      continue
    }

    const rel = path.relative(directory, file).replace(/\\/g, "/")
    if (includeSecrets) {
      findings.push(...scanSecrets(rel, content))
    }
    if (includeRules) {
      findings.push(...scanRules(rel, content, rules))
    }
  }

  const finishedAt = new Date().toISOString()
  return {
    startedAt,
    finishedAt,
    directory,
    filesScanned: files.length,
    findings,
    summary: summarize(findings),
  }
}

async function listSourceFiles(root: string, maxFiles: number): Promise<string[]> {
  const out: string[] = []
  async function walk(dir: string) {
    if (out.length >= maxFiles) return
    let entries
    try {
      entries = await fs.readdir(dir, { withFileTypes: true })
    } catch {
      return
    }
    for (const entry of entries) {
      if (out.length >= maxFiles) return
      const name = entry.name
      if (name.startsWith(".") && name !== ".env" && name !== ".env.local" && name !== ".env.example") {
        if (entry.isDirectory()) continue
      }
      if (entry.isDirectory()) {
        if (SKIP_DIRS.has(name)) continue
        await walk(path.join(dir, name))
        continue
      }
      if (!entry.isFile()) continue
      const ext = path.extname(name).toLowerCase()
      if (SKIP_EXT.has(ext)) continue
      if (name.endsWith(".min.js") || name.endsWith(".min.css")) continue
      out.push(path.join(dir, name))
    }
  }
  await walk(root)
  return out
}

function scanSecrets(relPath: string, content: string): Finding[] {
  const findings: Finding[] = []
  const lines = content.split(/\r?\n/)
  for (const pat of SECRET_PATTERNS) {
    // Reset lastIndex for global regexes
    pat.pattern.lastIndex = 0
    let match: RegExpExecArray | null
    const re = new RegExp(pat.pattern.source, pat.pattern.flags.includes("g") ? pat.pattern.flags : pat.pattern.flags + "g")
    while ((match = re.exec(content)) !== null) {
      const full = match[0]
      const captured = match[1] ?? full
      const before = content.slice(0, match.index)
      const line = before.split(/\r?\n/).length
      const snippet = lines[line - 1]?.trim().slice(0, 200) ?? ""
      // Skip obvious placeholders
      if (/your[_-]?api[_-]?key|xxx+|placeholder|example|changeme|todo/i.test(captured)) continue
      findings.push(
        makeFinding({
          ruleId: pat.id,
          title: pat.title,
          message: `Possible secret detected (${pat.title}). Value redacted.`,
          severity: pat.severity,
          path: relPath,
          line,
          snippet: snippet.replace(captured, pat.redact(captured)),
          tags: ["secrets"],
          source: "secrets",
          evidence: [`Redacted match: ${pat.redact(captured)}`, `Line ${line} in ${relPath}`],
        }),
      )
      if (findings.length > 200) return findings
    }
  }
  return findings
}

function scanRules(relPath: string, content: string, rules: Rule[]): Finding[] {
  const findings: Finding[] = []
  const lines = content.split(/\r?\n/)
  for (const rule of rules) {
    if (!pathMatchesRule(relPath, rule)) continue
    let re: RegExp
    try {
      // Case-insensitive by default — rule packs use keyword heuristics (SQL, crypto, …)
      re = new RegExp(rule.pattern, "gi")
    } catch {
      continue
    }
    let match: RegExpExecArray | null
    while ((match = re.exec(content)) !== null) {
      const before = content.slice(0, match.index)
      const line = before.split(/\r?\n/).length
      const snippet = lines[line - 1]?.trim().slice(0, 200) ?? ""
      findings.push(
        makeFinding({
          ruleId: rule.id,
          title: rule.title,
          message: rule.message,
          severity: rule.severity,
          path: relPath,
          line,
          snippet,
          cwe: rule.cwe,
          tags: rule.tags ?? [],
          source: "rules",
          evidence: [
            `Rule ${rule.id} matched at ${relPath}:${line}`,
            rule.cwe ? `Mapped to ${rule.cwe}` : "Heuristic match — verify before confirming",
            `Snippet: ${snippet}`,
          ],
        }),
      )
      if (findings.length > 500) return findings
    }
  }
  return findings
}

function makeFinding(input: {
  ruleId: string
  title: string
  message: string
  severity: Severity
  path: string
  line: number
  snippet: string
  cwe?: string
  tags: string[]
  source: Finding["source"]
  evidence: string[]
}): Finding {
  const now = new Date().toISOString()
  const stable = createHash("sha1")
    .update(`${input.ruleId}|${input.path}|${input.line}|${input.title}`)
    .digest("hex")
    .slice(0, 12)
  return {
    id: `fnd_${stable}`,
    ruleId: input.ruleId,
    title: input.title,
    message: input.message,
    severity: input.severity,
    path: input.path,
    line: input.line,
    snippet: input.snippet,
    cwe: input.cwe,
    tags: input.tags,
    status: "draft",
    evidence: input.evidence,
    source: input.source,
    createdAt: now,
    updatedAt: now,
  }
}

function summarize(findings: Finding[]): Record<Severity, number> {
  const summary: Record<Severity, number> = {
    critical: 0,
    high: 0,
    medium: 0,
    low: 0,
    info: 0,
  }
  for (const f of findings) {
    if (f.status === "false_positive" || f.status === "fixed") continue
    summary[f.severity]++
  }
  return summary
}

/** Generate a stable-looking UUID for external IDs when needed */
export function newId(): string {
  return randomUUID()
}
