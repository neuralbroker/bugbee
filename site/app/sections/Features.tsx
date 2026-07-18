"use client";

import { useEffect, useState } from "react";
import {
  Activity,
  GitBranch,
  LayoutDashboard,
  Settings2,
  Shield,
  Wand2,
} from "lucide-react";
import { FadeIn } from "@/app/components/animations/FadeIn";
import { cn } from "@/app/lib/utils";

function Waveform() {
  const [bars, setBars] = useState<number[]>(() =>
    Array.from({ length: 32 }, () => 0.3)
  );

  useEffect(() => {
    const id = setInterval(() => {
      setBars((prev) => prev.map(() => 0.2 + Math.random() * 0.8));
    }, 120);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="mt-6 flex h-24 items-end gap-1">
      {bars.map((h, i) => (
        <div
          key={i}
          className="flex-1 rounded-sm bg-gradient-to-t from-primary/80 to-accent/60 transition-all duration-150"
          style={{ height: `${h * 100}%` }}
        />
      ))}
    </div>
  );
}

function MiniChart() {
  return (
    <svg viewBox="0 0 200 80" className="mt-4 h-20 w-full" aria-hidden>
      <defs>
        <linearGradient id="chartFill" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#f59e0b" stopOpacity="0.4" />
          <stop offset="100%" stopColor="#f59e0b" stopOpacity="0" />
        </linearGradient>
      </defs>
      <path
        d="M0 60 L20 55 L40 50 L60 35 L80 40 L100 25 L120 30 L140 15 L160 20 L180 10 L200 12 L200 80 L0 80 Z"
        fill="url(#chartFill)"
      />
      <path
        d="M0 60 L20 55 L40 50 L60 35 L80 40 L100 25 L120 30 L140 15 L160 20 L180 10 L200 12"
        fill="none"
        stroke="#f59e0b"
        strokeWidth="2"
      />
    </svg>
  );
}

const DIFF = {
  before: `const token = req.headers.auth;`,
  after: `const token = req.headers.authorization;
if (!token) throw new AuthError();`,
};

export function Features() {
  return (
    <section id="features" className="section-pad relative">
      <div className="container-x">
        <FadeIn className="mx-auto max-w-2xl text-center">
          <p className="mb-4 text-xs font-semibold uppercase tracking-[0.2em] text-primary">
            Features
          </p>
          <h2 className="font-display text-[clamp(2rem,4.5vw,3.5rem)] font-semibold leading-[1.1] tracking-[-0.02em] text-white">
            One platform. Zero bugs.
          </h2>
          <p className="mt-5 text-base text-muted sm:text-lg">
            Everything your team needs to ship secure code — without leaving the
            PR.
          </p>
        </FadeIn>

        <div className="mt-14 grid auto-rows-fr gap-4 md:grid-cols-4 md:grid-rows-2">
          {/* Real-time — spans 2 cols */}
          <FadeIn
            delay={0.05}
            className="conic-border rounded-2xl md:col-span-2"
          >
            <div className="glass-card gradient-border h-full rounded-2xl p-6 transition-shadow hover:shadow-glow-sm">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/15 text-primary">
                  <Activity className="h-5 w-5" />
                </div>
                <div>
                  <h3 className="text-lg font-semibold text-white">
                    Real-Time Detection
                  </h3>
                  <p className="text-sm text-muted">
                    Scans on every push and PR open
                  </p>
                </div>
              </div>
              <Waveform />
            </div>
          </FadeIn>

          {/* Smart Fixes — spans 2 rows */}
          <FadeIn
            delay={0.1}
            className="conic-border rounded-2xl md:col-span-2 md:row-span-2"
          >
            <div className="glass-card gradient-border flex h-full flex-col rounded-2xl p-6 transition-shadow hover:shadow-glow-sm">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-secondary/15 text-secondary">
                  <Wand2 className="h-5 w-5" />
                </div>
                <div>
                  <h3 className="text-lg font-semibold text-white">
                    Smart Fixes
                  </h3>
                  <p className="text-sm text-muted">
                    AI patches you can approve in one click
                  </p>
                </div>
              </div>
              <div className="mt-6 flex flex-1 flex-col gap-3 font-mono text-xs">
                <div className="rounded-lg border border-error/20 bg-error/5 p-3">
                  <span className="mb-1 block text-[10px] font-semibold uppercase text-error">
                    − Before
                  </span>
                  <code className="text-white/70">{DIFF.before}</code>
                </div>
                <div className="rounded-lg border border-success/20 bg-success/5 p-3">
                  <span className="mb-1 block text-[10px] font-semibold uppercase text-success">
                    + After
                  </span>
                  <pre className="whitespace-pre-wrap text-white/85">
                    {DIFF.after}
                  </pre>
                </div>
              </div>
            </div>
          </FadeIn>

          <FeatureTile
            icon={GitBranch}
            title="CI/CD Integration"
            body="GitHub, GitLab, Bitbucket — status checks that block risky merges."
            color="text-accent"
            bg="bg-accent/15"
            delay={0.15}
          />
          <FeatureTile
            icon={Shield}
            title="Security Scanning"
            body="OWASP, CVE, secrets, taint analysis — defensive only, zero exploits."
            color="text-primary"
            bg="bg-primary/15"
            delay={0.2}
          />
          <FeatureTile
            icon={LayoutDashboard}
            title="Team Dashboard"
            body="Trends, hotspots, and ownership so leads see risk before standup."
            color="text-secondary"
            bg="bg-secondary/15"
            delay={0.25}
            chart
          />
          <FeatureTile
            icon={Settings2}
            title="Custom Rules"
            body="Encode your standards once. Enforce them on every commit."
            color="text-accent"
            bg="bg-accent/15"
            delay={0.3}
            spin
          />
        </div>
      </div>
    </section>
  );
}

function FeatureTile({
  icon: Icon,
  title,
  body,
  color,
  bg,
  delay,
  chart,
  spin,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  body: string;
  color: string;
  bg: string;
  delay: number;
  chart?: boolean;
  spin?: boolean;
}) {
  return (
    <FadeIn delay={delay} className="conic-border rounded-2xl">
      <div className="glass-card gradient-border group h-full rounded-2xl p-5 transition-shadow hover:shadow-glow-sm">
        <div
          className={cn(
            "mb-4 inline-flex h-10 w-10 items-center justify-center rounded-xl transition-colors",
            bg,
            color
          )}
        >
          <Icon
            className={cn(
              "h-5 w-5 transition-transform duration-500 group-hover:scale-110",
              spin && "group-hover:rotate-90"
            )}
          />
        </div>
        <h3 className="text-base font-semibold text-white">{title}</h3>
        <p className="mt-2 text-sm leading-relaxed text-muted">{body}</p>
        {chart && <MiniChart />}
      </div>
    </FadeIn>
  );
}
