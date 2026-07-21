import type { Rule } from "./types"

/** Built-in defensive rule pack (OWASP-inspired heuristics). Evidence-first; no live exploits. */
export const BUILTIN_RULES: Rule[] = [
  {
    id: "owasp.python.eval",
    title: "Dangerous eval()",
    message: "Dynamic eval of untrusted input can lead to RCE.",
    severity: "critical",
    pattern: String.raw`\beval\s*\(`,
    paths: ["*.py"],
    cwe: "CWE-95",
    tags: ["injection", "rce"],
  },
  {
    id: "owasp.python.exec",
    title: "Dangerous exec()",
    message: "Dynamic exec of untrusted input can lead to RCE.",
    severity: "critical",
    pattern: String.raw`\bexec\s*\(`,
    paths: ["*.py"],
    cwe: "CWE-95",
    tags: ["injection", "rce"],
  },
  {
    id: "owasp.js.eval",
    title: "Dangerous eval()",
    message: "eval() on untrusted data enables code injection.",
    severity: "critical",
    pattern: String.raw`\beval\s*\(`,
    paths: ["*.js", "*.ts", "*.jsx", "*.tsx", "*.mjs", "*.cjs"],
    cwe: "CWE-95",
    tags: ["injection"],
  },
  {
    id: "owasp.js.function_constructor",
    title: "Function constructor",
    message: "new Function() is an eval-equivalent injection sink.",
    severity: "high",
    pattern: String.raw`new\s+Function\s*\(`,
    paths: ["*.js", "*.ts", "*.jsx", "*.tsx", "*.mjs", "*.cjs"],
    cwe: "CWE-95",
    tags: ["injection"],
  },
  {
    id: "owasp.sql.concatenate",
    title: "Possible SQL string concatenation",
    message: "Building SQL with string concatenation often causes injection.",
    severity: "high",
    pattern: String.raw`(SELECT|INSERT|UPDATE|DELETE).{0,80}(\+|format\(|f["'])`,
    paths: ["*.py", "*.js", "*.ts", "*.go", "*.php", "*.java", "*.rb"],
    cwe: "CWE-89",
    tags: ["sqli"],
  },
  {
    id: "owasp.command.shell_true",
    title: "shell=True in subprocess",
    message: "subprocess with shell=True is a common command-injection sink.",
    severity: "high",
    pattern: String.raw`shell\s*=\s*True`,
    paths: ["*.py"],
    cwe: "CWE-78",
    tags: ["command-injection"],
  },
  {
    id: "owasp.command.os_system",
    title: "os.system() usage",
    message: "os.system() is a high-risk shell execution sink.",
    severity: "high",
    pattern: String.raw`\bos\.system\s*\(`,
    paths: ["*.py"],
    cwe: "CWE-78",
    tags: ["command-injection"],
  },
  {
    id: "owasp.php.raw_query",
    title: "Possible raw SQL in PHP",
    message: "Raw query construction may be injectable.",
    severity: "high",
    pattern: String.raw`(mysqli_query|mysql_query|->query)\s*\(`,
    paths: ["*.php"],
    cwe: "CWE-89",
    tags: ["sqli"],
  },
  {
    id: "owasp.ssrf.urlopen",
    title: "Server-side request heuristic",
    message: "Fetching remote URLs without allowlists can enable SSRF.",
    severity: "medium",
    pattern: String.raw`(urllib\.request\.urlopen|requests\.(get|post)|fetch\()\s*\(`,
    paths: ["*.py", "*.js", "*.ts", "*.tsx", "*.jsx"],
    cwe: "CWE-918",
    tags: ["ssrf"],
  },
  {
    id: "owasp.crypto.md5",
    title: "Weak hash MD5",
    message: "MD5 is not suitable for integrity or password storage.",
    severity: "medium",
    pattern: String.raw`\bmd5\b`,
    paths: ["*.py", "*.js", "*.ts", "*.go", "*.java", "*.rs", "*.php", "*.rb"],
    cwe: "CWE-328",
    tags: ["crypto"],
  },
  {
    id: "owasp.crypto.sha1",
    title: "Weak hash SHA1",
    message: "SHA-1 is cryptographically broken for collision-resistant use cases.",
    severity: "low",
    pattern: String.raw`\bsha1\b`,
    paths: ["*.py", "*.js", "*.ts", "*.go", "*.java", "*.rs", "*.php"],
    cwe: "CWE-328",
    tags: ["crypto"],
  },
  {
    id: "owasp.path.traversal",
    title: "Path join with user input heuristic",
    message: "Joining untrusted input into filesystem paths can enable path traversal.",
    severity: "medium",
    pattern: String.raw`(path\.join|os\.path\.join|filepath\.Join)\s*\([^)]*(req\.|request\.|params\.|query\.|body\.)`,
    paths: ["*.js", "*.ts", "*.py", "*.go"],
    cwe: "CWE-22",
    tags: ["path-traversal"],
  },
  {
    id: "owasp.xss.innerhtml",
    title: "innerHTML assignment",
    message: "Assigning untrusted data to innerHTML can enable XSS.",
    severity: "high",
    pattern: String.raw`\.innerHTML\s*=`,
    paths: ["*.js", "*.ts", "*.jsx", "*.tsx", "*.vue", "*.svelte"],
    cwe: "CWE-79",
    tags: ["xss"],
  },
  {
    id: "owasp.auth.hardcoded_password",
    title: "Hardcoded password assignment",
    message: "Hardcoded credentials in source should be rotated and externalized.",
    severity: "critical",
    pattern: String.raw`(password|passwd|pwd)\s*[:=]\s*['"][^'"]{4,}['"]`,
    paths: ["*.py", "*.js", "*.ts", "*.go", "*.java", "*.rb", "*.php", "*.rs", "*.env"],
    cwe: "CWE-798",
    tags: ["secrets", "auth"],
  },
  {
    id: "owasp.deser.pickle",
    title: "Unsafe pickle loads",
    message: "pickle.loads on untrusted data enables arbitrary code execution.",
    severity: "critical",
    pattern: String.raw`\bpickle\.loads?\s*\(`,
    paths: ["*.py"],
    cwe: "CWE-502",
    tags: ["deserialization"],
  },
  {
    id: "owasp.deser.yaml_load",
    title: "Unsafe yaml.load",
    message: "yaml.load without SafeLoader can execute arbitrary constructors.",
    severity: "high",
    pattern: String.raw`\byaml\.load\s*\(`,
    paths: ["*.py"],
    cwe: "CWE-502",
    tags: ["deserialization"],
  },
  {
    id: "owasp.go.unsafe",
    title: "Go unsafe package usage",
    message: "unsafe package bypasses memory safety; review carefully.",
    severity: "medium",
    pattern: String.raw`"unsafe"`,
    paths: ["*.go"],
    cwe: "CWE-119",
    tags: ["memory"],
  },
  {
    id: "owasp.cors.wildcard",
    title: "CORS Access-Control-Allow-Origin: *",
    message: "Wildcard CORS on credentialed APIs can leak user data.",
    severity: "medium",
    pattern: String.raw`Access-Control-Allow-Origin['"\s:=*]*\*`,
    paths: ["*.js", "*.ts", "*.py", "*.go", "*.java", "*.rb", "*.php"],
    cwe: "CWE-942",
    tags: ["cors"],
  },
]

export function pathMatchesRule(filePath: string, rule: Rule): boolean {
  if (!rule.paths || rule.paths.length === 0) return true
  const base = filePath.replace(/\\/g, "/")
  const name = base.split("/").pop() ?? base
  return rule.paths.some((glob) => {
    // Simple glob: *.ext or *.{a,b}
    if (glob.startsWith("*.{") && glob.endsWith("}")) {
      const inner = glob.slice(3, -1)
      const exts = inner.split(",").map((e) => e.trim())
      return exts.some((ext) => name.endsWith(`.${ext}`) || name.endsWith(ext.replace(/^\./, "")))
    }
    if (glob.startsWith("*.")) {
      const ext = glob.slice(1) // .py
      return name.endsWith(ext)
    }
    return name === glob || base.endsWith(glob)
  })
}
