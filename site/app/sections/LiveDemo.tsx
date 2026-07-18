"use client";

import { useMemo, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import {
  AlertTriangle,
  Check,
  Info,
  Loader2,
  Play,
  XCircle,
} from "lucide-react";
import { FadeIn } from "@/app/components/animations/FadeIn";
import { Button } from "@/app/components/ui/Button";
import { cn } from "@/app/lib/utils";
import type { BugFinding, Severity } from "@/app/types";

type Lang = "js" | "python" | "rust";

const CODE: Record<Lang, { label: string; lines: string[] }> = {
  js: {
    label: "auth.js",
    lines: [
      "export async function login(req, res) {",
      "  const { email, password } = req.body;",
      "  const user = await db.query(",
      "    `SELECT * FROM users WHERE email = '${email}'`",
      "  );",
      "  if (user && user.password === password) {",
      "    res.cookie('session', user.id);",
      "    return res.json({ ok: true });",
      "  }",
      "  res.status(401).send('fail');",
      "}",
    ],
  },
  python: {
    label: "auth.py",
    lines: [
      "def login(email, password):",
      "    q = f\"SELECT * FROM users WHERE email = '{email}'\"",
      "    user = db.execute(q).fetchone()",
      "    if user and user['password'] == password:",
      "        session['uid'] = user['id']",
      "        return {'ok': True}",
      "    return {'error': 'fail'}, 401",
    ],
  },
  rust: {
    label: "auth.rs",
    lines: [
      "fn login(email: &str, password: &str) -> Result<User> {",
      "    let q = format!(",
      "        \"SELECT * FROM users WHERE email = '{}'\", email",
      "    );",
      "    let user = db.query(&q)?;",
      "    if user.password == password {",
      "        Ok(user)",
      "    } else {",
      "        Err(AuthError::Invalid)",
      "    }",
      "}",
    ],
  },
};

const FINDINGS: Record<Lang, BugFinding[]> = {
  js: [
    {
      id: "js-1",
      line: 4,
      severity: "critical",
      title: "SQL Injection",
      explanation:
        "User-controlled `email` is interpolated into a raw SQL string. An attacker can alter the query and bypass auth or dump data.",
      fix: "Use a parameterized query with bound parameters.",
      original: "`SELECT * FROM users WHERE email = '${email}'`",
      replacement: "\"SELECT * FROM users WHERE email = $1\", [email]",
    },
    {
      id: "js-2",
      line: 6,
      severity: "critical",
      title: "Plaintext password compare",
      explanation:
        "Passwords should never be stored or compared in plaintext. Use bcrypt/argon2 with constant-time compare.",
      fix: "Hash passwords at rest; compare with crypto.timingSafeEqual or lib verify.",
      original: "user.password === password",
      replacement: "await bcrypt.compare(password, user.passwordHash)",
    },
    {
      id: "js-3",
      line: 7,
      severity: "warning",
      title: "Insecure session cookie",
      explanation:
        "Session cookie lacks HttpOnly, Secure, and SameSite flags — vulnerable to XSS theft.",
      fix: "Set secure cookie options.",
      original: "res.cookie('session', user.id)",
      replacement:
        "res.cookie('session', token, { httpOnly: true, secure: true, sameSite: 'lax' })",
    },
  ],
  python: [
    {
      id: "py-1",
      line: 2,
      severity: "critical",
      title: "SQL Injection (f-string)",
      explanation:
        "f-string SQL construction is a classic injection vector. Never format user input into queries.",
      fix: "Use parameterized execute with placeholders.",
      original: "f\"SELECT * FROM users WHERE email = '{email}'\"",
      replacement: "\"SELECT * FROM users WHERE email = %s\", (email,)",
    },
    {
      id: "py-2",
      line: 4,
      severity: "critical",
      title: "Plaintext password",
      explanation: "Direct password equality is unsafe. Use a password hashing library.",
      fix: "Store password hashes; verify with passlib or argon2.",
      original: "user['password'] == password",
      replacement: "pwd_context.verify(password, user['password_hash'])",
    },
  ],
  rust: [
    {
      id: "rs-1",
      line: 3,
      severity: "critical",
      title: "SQL Injection via format!",
      explanation:
        "format! into SQL is injection-prone. Use a query builder or prepared statements.",
      fix: "Use sqlx query! macros with bound params.",
      original: "format!(\"SELECT * FROM users WHERE email = '{}'\", email)",
      replacement: "sqlx::query_as!(User, \"SELECT * FROM users WHERE email = $1\", email)",
    },
    {
      id: "rs-2",
      line: 6,
      severity: "warning",
      title: "Timing-unsafe password compare",
      explanation:
        "String equality can leak timing information. Use constant-time comparison.",
      fix: "Use subtle::ConstantTimeEq or argon2 verify.",
      original: "user.password == password",
      replacement: "argon2::verify_encoded(&user.hash, password.as_bytes())?",
    },
  ],
};

const severityIcon: Record<Severity, React.ReactNode> = {
  critical: <XCircle className="h-3.5 w-3.5 text-error" />,
  warning: <AlertTriangle className="h-3.5 w-3.5 text-primary" />,
  info: <Info className="h-3.5 w-3.5 text-secondary" />,
};

const severityColor: Record<Severity, string> = {
  critical: "bg-error/20 border-error/40 text-error",
  warning: "bg-primary/20 border-primary/40 text-primary",
  info: "bg-secondary/20 border-secondary/40 text-secondary",
};

type Phase = "idle" | "scanning" | "results" | "fixed";

export function LiveDemo() {
  const [lang, setLang] = useState<Lang>("js");
  const [phase, setPhase] = useState<Phase>("idle");
  const [progress, setProgress] = useState(0);
  const [visibleBugs, setVisibleBugs] = useState<string[]>([]);
  const [selected, setSelected] = useState<BugFinding | null>(null);
  const [applied, setApplied] = useState<Set<string>>(new Set());

  const code = CODE[lang];
  const findings = FINDINGS[lang];

  const lineBugMap = useMemo(() => {
    const map = new Map<number, BugFinding>();
    findings.forEach((f) => map.set(f.line, f));
    return map;
  }, [findings]);

  const runAnalysis = async () => {
    setPhase("scanning");
    setProgress(0);
    setVisibleBugs([]);
    setSelected(null);
    setApplied(new Set());

    for (let p = 0; p <= 100; p += 4) {
      await new Promise((r) => setTimeout(r, 40));
      setProgress(p);
    }

    setPhase("results");
    for (const f of findings) {
      await new Promise((r) => setTimeout(r, 350));
      setVisibleBugs((v) => [...v, f.id]);
    }
  };

  const applyFix = (bug: BugFinding) => {
    setApplied((s) => new Set(s).add(bug.id));
    setSelected(null);
    if (applied.size + 1 >= findings.length) {
      setPhase("fixed");
    }
  };

  const switchLang = (l: Lang) => {
    setLang(l);
    setPhase("idle");
    setVisibleBugs([]);
    setSelected(null);
    setApplied(new Set());
    setProgress(0);
  };

  return (
    <section id="demo" className="section-pad relative">
      <div className="container-x">
        <FadeIn className="mx-auto max-w-2xl text-center">
          <p className="mb-4 text-xs font-semibold uppercase tracking-[0.2em] text-primary">
            Live playground
          </p>
          <h2 className="font-display text-[clamp(2rem,4.5vw,3.5rem)] font-semibold leading-[1.1] tracking-[-0.02em] text-white">
            See BugBee in action.
          </h2>
          <p className="mt-5 text-base text-muted sm:text-lg">
            Pick a language, run analysis, and apply AI fixes — all simulated
            in your browser. No backend required.
          </p>
        </FadeIn>

        <FadeIn delay={0.15} className="mt-12">
          <div className="overflow-hidden rounded-2xl border border-white/10 bg-surface shadow-card">
            {/* Tabs + run */}
            <div className="flex flex-wrap items-center justify-between gap-3 border-b border-white/[0.06] px-4 py-3">
              <div className="flex gap-1">
                {(Object.keys(CODE) as Lang[]).map((l) => (
                  <button
                    key={l}
                    type="button"
                    onClick={() => switchLang(l)}
                    className={cn(
                      "rounded-lg px-3 py-1.5 font-mono text-xs font-medium transition-colors",
                      lang === l
                        ? "bg-primary/15 text-primary"
                        : "text-muted hover:text-white"
                    )}
                  >
                    {l === "js" ? "JavaScript" : l === "python" ? "Python" : "Rust"}
                  </button>
                ))}
              </div>
              <Button
                size="sm"
                onClick={runAnalysis}
                disabled={phase === "scanning"}
                className="gap-2"
              >
                {phase === "scanning" ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Scanning…
                  </>
                ) : (
                  <>
                    <Play className="h-4 w-4" />
                    Run Analysis
                  </>
                )}
              </Button>
            </div>

            {phase === "scanning" && (
              <div className="h-1 w-full bg-white/5">
                <div
                  className="h-full bg-gradient-to-r from-primary to-secondary transition-all duration-100"
                  style={{ width: `${progress}%` }}
                />
              </div>
            )}

            <div className="grid lg:grid-cols-[1fr_320px]">
              {/* Editor */}
              <div className="min-h-[320px] overflow-x-auto p-4 font-mono text-[13px] leading-relaxed">
                <p className="mb-3 text-[11px] text-muted">{code.label}</p>
                {code.lines.map((line, i) => {
                  const lineNo = i + 1;
                  const bug = lineBugMap.get(lineNo);
                  const isVisible = bug && visibleBugs.includes(bug.id);
                  const isApplied = bug && applied.has(bug.id);

                  return (
                    <div
                      key={i}
                      className={cn(
                        "flex gap-4 rounded px-1 transition-colors",
                        isVisible && !isApplied && "bg-error/10",
                        isApplied && "bg-success/10"
                      )}
                    >
                      <span className="w-6 shrink-0 select-none text-right text-white/25">
                        {lineNo}
                      </span>
                      <button
                        type="button"
                        disabled={!isVisible || isApplied}
                        onClick={() => bug && setSelected(bug)}
                        className={cn(
                          "flex-1 text-left text-white/80",
                          isVisible &&
                            !isApplied &&
                            "underline decoration-error decoration-2 underline-offset-4 cursor-pointer",
                          isApplied && "text-success/90 no-underline"
                        )}
                      >
                        {isApplied && bug
                          ? bug.replacement.split("\n")[0]
                          : line}
                      </button>
                      {isVisible && !isApplied && bug && (
                        <span
                          className={cn(
                            "flex shrink-0 items-center gap-1 rounded border px-1.5 text-[10px] font-semibold uppercase",
                            severityColor[bug.severity]
                          )}
                        >
                          {severityIcon[bug.severity]}
                          {bug.severity}
                        </span>
                      )}
                      {isApplied && (
                        <Check className="h-4 w-4 shrink-0 text-success" />
                      )}
                    </div>
                  );
                })}
              </div>

              {/* Side panel */}
              <div className="border-t border-white/[0.06] bg-surface-elevated/50 p-4 lg:border-l lg:border-t-0">
                <h4 className="text-sm font-semibold text-white">
                  Findings
                  {phase === "results" || phase === "fixed"
                    ? ` (${visibleBugs.length})`
                    : ""}
                </h4>

                <AnimatePresence mode="wait">
                  {selected ? (
                    <motion.div
                      key={selected.id}
                      initial={{ opacity: 0, x: 16 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: 16 }}
                      className="mt-4 space-y-3"
                    >
                      <div
                        className={cn(
                          "inline-flex items-center gap-1.5 rounded border px-2 py-0.5 text-[10px] font-semibold uppercase",
                          severityColor[selected.severity]
                        )}
                      >
                        {severityIcon[selected.severity]}
                        {selected.severity}
                      </div>
                      <h5 className="font-semibold text-white">
                        {selected.title}
                      </h5>
                      <p className="text-sm leading-relaxed text-muted">
                        {selected.explanation}
                      </p>
                      <div className="rounded-lg border border-secondary/25 bg-secondary/10 p-3">
                        <p className="mb-1 text-[10px] font-semibold uppercase text-secondary">
                          Suggested fix
                        </p>
                        <p className="text-sm text-white/90">{selected.fix}</p>
                        <pre className="mt-2 overflow-x-auto rounded bg-void/50 p-2 font-mono text-[11px] text-accent">
                          {selected.replacement}
                        </pre>
                      </div>
                      <Button
                        size="sm"
                        className="w-full"
                        onClick={() => applyFix(selected)}
                      >
                        Apply Fix
                      </Button>
                      <button
                        type="button"
                        className="w-full text-center text-xs text-muted hover:text-white"
                        onClick={() => setSelected(null)}
                      >
                        Back to list
                      </button>
                    </motion.div>
                  ) : (
                    <motion.div
                      key="list"
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      className="mt-4 space-y-2"
                    >
                      {phase === "idle" && (
                        <p className="text-sm text-muted">
                          Hit <strong className="text-white">Run Analysis</strong>{" "}
                          to scan this snippet for vulnerabilities.
                        </p>
                      )}
                      {phase === "scanning" && (
                        <p className="flex items-center gap-2 text-sm text-muted">
                          <Loader2 className="h-4 w-4 animate-spin text-primary" />
                          Neural engine scanning…
                        </p>
                      )}
                      {phase === "fixed" && (
                        <div className="rounded-lg border border-success/30 bg-success/10 p-3 text-sm text-success">
                          All findings fixed. Ship with confidence.
                        </div>
                      )}
                      {findings
                        .filter((f) => visibleBugs.includes(f.id))
                        .map((f) => (
                          <button
                            key={f.id}
                            type="button"
                            onClick={() =>
                              !applied.has(f.id) && setSelected(f)
                            }
                            className={cn(
                              "flex w-full items-start gap-2 rounded-lg border border-white/[0.06] bg-white/[0.03] p-3 text-left transition-colors hover:bg-white/[0.06]",
                              applied.has(f.id) && "opacity-50"
                            )}
                          >
                            {applied.has(f.id) ? (
                              <Check className="mt-0.5 h-4 w-4 text-success" />
                            ) : (
                              severityIcon[f.severity]
                            )}
                            <div>
                              <p className="text-sm font-medium text-white">
                                {f.title}
                              </p>
                              <p className="text-[11px] text-muted">
                                Line {f.line}
                              </p>
                            </div>
                          </button>
                        ))}
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            </div>
          </div>
        </FadeIn>
      </div>
    </section>
  );
}
