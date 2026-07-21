import type { Severity } from "./types"

export interface SecretPattern {
  id: string
  title: string
  severity: Severity
  pattern: RegExp
  redact: (match: string) => string
}

function redactMiddle(value: string, keep = 4): string {
  if (value.length <= keep * 2) return "*".repeat(value.length)
  return `${value.slice(0, keep)}${"*".repeat(Math.min(12, value.length - keep * 2))}${value.slice(-keep)}`
}

/** High-signal secret detectors. Never echo full secrets in findings. */
export const SECRET_PATTERNS: SecretPattern[] = [
  {
    id: "secret.aws_access_key",
    title: "AWS Access Key ID",
    severity: "critical",
    pattern: /\b(AKIA[0-9A-Z]{16})\b/g,
    redact: (m) => redactMiddle(m),
  },
  {
    id: "secret.aws_secret_key",
    title: "AWS Secret Access Key heuristic",
    severity: "critical",
    pattern: /(?:aws_secret_access_key|AWS_SECRET_ACCESS_KEY)\s*[:=]\s*['"]?([A-Za-z0-9/+=]{40})['"]?/g,
    redact: (m) => redactMiddle(m, 3),
  },
  {
    id: "secret.github_pat",
    title: "GitHub personal access token",
    severity: "critical",
    pattern: /\b(ghp_[A-Za-z0-9]{36,})\b/g,
    redact: (m) => redactMiddle(m),
  },
  {
    id: "secret.github_oauth",
    title: "GitHub OAuth token",
    severity: "critical",
    pattern: /\b(gho_[A-Za-z0-9]{36,})\b/g,
    redact: (m) => redactMiddle(m),
  },
  {
    id: "secret.slack_token",
    title: "Slack token",
    severity: "high",
    pattern: /\b(xox[baprs]-[A-Za-z0-9-]{10,})\b/g,
    redact: (m) => redactMiddle(m),
  },
  {
    id: "secret.stripe_live",
    title: "Stripe live secret key",
    severity: "critical",
    pattern: /\b(sk_live_[A-Za-z0-9]{20,})\b/g,
    redact: (m) => redactMiddle(m),
  },
  {
    id: "secret.openai_key",
    title: "OpenAI API key",
    severity: "critical",
    pattern: /\b(sk-[A-Za-z0-9]{20,})\b/g,
    redact: (m) => redactMiddle(m),
  },
  {
    id: "secret.anthropic_key",
    title: "Anthropic API key",
    severity: "critical",
    pattern: /\b(sk-ant-[A-Za-z0-9\-_]{20,})\b/g,
    redact: (m) => redactMiddle(m),
  },
  {
    id: "secret.google_api",
    title: "Google API key",
    severity: "high",
    pattern: /\b(AIza[0-9A-Za-z\-_]{35})\b/g,
    redact: (m) => redactMiddle(m),
  },
  {
    id: "secret.private_key",
    title: "Private key block",
    severity: "critical",
    pattern: /-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----/g,
    redact: () => "-----BEGIN PRIVATE KEY----- [REDACTED]",
  },
  {
    id: "secret.jwt",
    title: "JWT token",
    severity: "medium",
    pattern: /\b(eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,})\b/g,
    redact: (m) => redactMiddle(m, 6),
  },
  {
    id: "secret.generic_api_key",
    title: "Generic API key assignment",
    severity: "medium",
    pattern: /(?:api[_-]?key|apikey|secret[_-]?key|access[_-]?token)\s*[:=]\s*['"]([A-Za-z0-9_\-]{16,})['"]/gi,
    redact: (m) => redactMiddle(m),
  },
]
