import fs from "fs/promises"
import path from "path"
import type { Finding, FindingsStore, FindingStatus } from "./types"

const STORE_DIR = ".bugbee"
const STORE_FILE = "findings.json"

export function storePath(directory: string): string {
  return path.join(directory, STORE_DIR, STORE_FILE)
}

export async function loadFindings(directory: string): Promise<FindingsStore> {
  const file = storePath(directory)
  try {
    const raw = await fs.readFile(file, "utf8")
    const parsed = JSON.parse(raw) as FindingsStore
    if (!parsed.findings || !Array.isArray(parsed.findings)) {
      return emptyStore()
    }
    return parsed
  } catch {
    return emptyStore()
  }
}

export function emptyStore(): FindingsStore {
  return { version: 1, updatedAt: new Date().toISOString(), findings: [] }
}

export async function saveFindings(directory: string, store: FindingsStore): Promise<string> {
  const file = storePath(directory)
  await fs.mkdir(path.dirname(file), { recursive: true })
  store.updatedAt = new Date().toISOString()
  await fs.writeFile(file, JSON.stringify(store, null, 2) + "\n", "utf8")
  return file
}

export async function mergeFindings(directory: string, incoming: Finding[]): Promise<FindingsStore> {
  const store = await loadFindings(directory)
  const byKey = new Map<string, Finding>()
  for (const f of store.findings) {
    byKey.set(findingKey(f), f)
  }
  for (const f of incoming) {
    const key = findingKey(f)
    const existing = byKey.get(key)
    if (existing) {
      byKey.set(key, {
        ...existing,
        ...f,
        id: existing.id,
        status: existing.status === "fixed" || existing.status === "false_positive" ? existing.status : f.status,
        createdAt: existing.createdAt,
        updatedAt: new Date().toISOString(),
      })
    } else {
      byKey.set(key, f)
    }
  }
  store.findings = Array.from(byKey.values()).sort((a, b) => severityRank(b.severity) - severityRank(a.severity))
  await saveFindings(directory, store)
  return store
}

export async function updateFindingStatus(
  directory: string,
  id: string,
  status: FindingStatus,
): Promise<Finding | undefined> {
  const store = await loadFindings(directory)
  const found = store.findings.find((f) => f.id === id)
  if (!found) return undefined
  found.status = status
  found.updatedAt = new Date().toISOString()
  await saveFindings(directory, store)
  return found
}

function findingKey(f: Finding): string {
  return `${f.ruleId}|${f.path}|${f.line}|${f.title}`
}

function severityRank(s: Finding["severity"]): number {
  switch (s) {
    case "critical":
      return 5
    case "high":
      return 4
    case "medium":
      return 3
    case "low":
      return 2
    default:
      return 1
  }
}
