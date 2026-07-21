export type Severity = "critical" | "high" | "medium" | "low" | "info"

export type FindingStatus = "draft" | "confirmed" | "false_positive" | "fixed"

export interface Rule {
  id: string
  title: string
  message: string
  severity: Severity
  pattern: string
  paths?: string[]
  cwe?: string
  tags?: string[]
}

export interface Finding {
  id: string
  ruleId: string
  title: string
  message: string
  severity: Severity
  path: string
  line: number
  column?: number
  snippet: string
  cwe?: string
  tags: string[]
  status: FindingStatus
  evidence: string[]
  source: "secrets" | "rules" | "agent"
  createdAt: string
  updatedAt: string
}

export interface HuntReport {
  startedAt: string
  finishedAt: string
  directory: string
  filesScanned: number
  findings: Finding[]
  summary: Record<Severity, number>
}

export interface FindingsStore {
  version: 1
  updatedAt: string
  findings: Finding[]
}
