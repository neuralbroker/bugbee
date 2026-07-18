"use client";

import dynamic from "next/dynamic";
import { useRef, useState } from "react";
import { motion } from "framer-motion";
import { ChevronDown, Play, Sparkles } from "lucide-react";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { Badge } from "@/app/components/ui/Badge";
import { Button } from "@/app/components/ui/Button";
import { useUIStore } from "@/app/store/ui";

const HeroBackground = dynamic(
  () =>
    import("@/app/components/three/HeroBackground").then(
      (m) => m.HeroBackground
    ),
  { ssr: false }
);

const SAMPLE_CODE = `async function processPayment(user, amount) {
  const query = \`SELECT * FROM users
    WHERE id = '\${user.id}'\`;  // ⚠ SQL injection
  const result = await db.query(query);

  // Missing auth check
  await chargeCard(user.card, amount);
  return { status: "ok", result };
}`;

const LOGOS = ["Vercel", "Stripe", "Linear", "Notion", "Figma"];

export function Hero() {
  const cardRef = useRef<HTMLDivElement>(null);
  const [tilt, setTilt] = useState({ x: 0, y: 0 });
  const setDemoModalOpen = useUIStore((s) => s.setDemoModalOpen);

  const onMove = (e: React.MouseEvent) => {
    const el = cardRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width - 0.5;
    const y = (e.clientY - rect.top) / rect.height - 0.5;
    setTilt({ x: y * -8, y: x * 10 });
  };

  return (
    <section
      id="top"
      className="relative flex min-h-[100svh] items-center overflow-hidden pt-24 pb-16"
    >
      <HeroBackground />

      <div className="container-x relative z-10 grid items-center gap-12 lg:grid-cols-[1.1fr_0.9fr] lg:gap-10">
        {/* Left */}
        <div className="max-w-xl">
          <motion.div
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.15 }}
          >
            <Badge pulse>
              <span aria-hidden>🐝</span>
              AI-Powered Bug Detection
            </Badge>
          </motion.div>

          <motion.h1
            initial={{ opacity: 0, y: 24 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.3 }}
            className="mt-6 font-display text-[clamp(2.75rem,6vw,5.5rem)] font-bold leading-[1.0] tracking-[-0.03em] text-white"
          >
            Catch bugs{" "}
            <span className="text-gradient-amber">before they bite.</span>
          </motion.h1>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.55, delay: 0.45 }}
            className="mt-6 max-w-lg text-base leading-relaxed text-muted sm:text-lg"
          >
            NeuralBroker&apos;s AI engine scans every commit, finds
            vulnerabilities, and suggests fixes — before your users do.
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.55 }}
            className="mt-8 flex flex-wrap items-center gap-3"
          >
            <Button href="#cta" variant="primary" size="lg" magnetic>
              Start Free
            </Button>
            <Button
              variant="ghost"
              size="lg"
              onClick={() => setDemoModalOpen(true)}
            >
              <Play className="h-4 w-4" />
              Watch Demo
            </Button>
          </motion.div>

          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.75 }}
            className="mt-10"
          >
            <p className="text-xs font-medium uppercase tracking-wider text-muted/80">
              Trusted by 12,000+ developers
            </p>
            <div className="mt-4 flex flex-wrap items-center gap-x-6 gap-y-2">
              {LOGOS.map((name) => (
                <span
                  key={name}
                  className="text-sm font-semibold tracking-tight text-white/30 transition-colors duration-300 hover:text-white/80"
                >
                  {name}
                </span>
              ))}
            </div>
          </motion.div>
        </div>

        {/* Right — floating code block */}
        <motion.div
          initial={{ opacity: 0, x: 30 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.7, delay: 0.4 }}
          className="relative mx-auto w-full max-w-md lg:max-w-none"
        >
          <div
            ref={cardRef}
            onMouseMove={onMove}
            onMouseLeave={() => setTilt({ x: 0, y: 0 })}
            style={{
              transform: `perspective(1000px) rotateX(${tilt.x}deg) rotateY(${tilt.y}deg)`,
              transition: "transform 0.15s ease-out",
            }}
            className="relative overflow-hidden rounded-2xl border border-white/10 bg-surface/90 shadow-card backdrop-blur-xl"
          >
            <div className="flex items-center gap-2 border-b border-white/[0.06] px-4 py-3">
              <span className="h-2.5 w-2.5 rounded-full bg-error/80" />
              <span className="h-2.5 w-2.5 rounded-full bg-primary/80" />
              <span className="h-2.5 w-2.5 rounded-full bg-success/80" />
              <span className="ml-2 font-mono text-[11px] text-muted">
                payment.ts — BugBee Live
              </span>
            </div>

            <div className="relative p-1">
              <SyntaxHighlighter
                language="typescript"
                style={oneDark}
                customStyle={{
                  margin: 0,
                  padding: "1rem 1.1rem",
                  background: "transparent",
                  fontSize: 12.5,
                  lineHeight: 1.65,
                }}
                showLineNumbers
                lineNumberStyle={{ color: "#475569", minWidth: "1.8em" }}
              >
                {SAMPLE_CODE}
              </SyntaxHighlighter>

              {/* Bug underline highlights */}
              <div className="pointer-events-none absolute left-[4.5rem] top-[4.6rem] h-0.5 w-[11rem] animate-pulse bg-error/80 shadow-[0_0_8px_#ef4444]" />
              <div className="pointer-events-none absolute left-[4.5rem] top-[7.8rem] h-0.5 w-[12rem] animate-pulse bg-primary/80 shadow-[0_0_8px_#f59e0b]" />
            </div>

            {/* AI suggestion tooltip */}
            <motion.div
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 1.2 }}
              className="m-3 rounded-xl border border-secondary/30 bg-secondary/10 p-3 backdrop-blur-sm"
            >
              <div className="mb-1.5 flex items-center gap-1.5 text-xs font-semibold text-secondary">
                <Sparkles className="h-3.5 w-3.5" />
                AI Fix Suggestion
              </div>
              <p className="font-mono text-[11px] leading-relaxed text-white/85">
                Use parameterized queries:{" "}
                <span className="text-accent">
                  db.query(&apos;SELECT * FROM users WHERE id = $1&apos;,
                  [user.id])
                </span>
              </p>
            </motion.div>
          </div>

          {/* Glow */}
          <div className="pointer-events-none absolute -inset-4 -z-10 rounded-3xl bg-primary/10 blur-3xl" />
        </motion.div>
      </div>

      <a
        href="#problem"
        className="absolute bottom-8 left-1/2 flex -translate-x-1/2 flex-col items-center gap-1 text-muted transition-colors hover:text-primary"
        aria-label="Scroll to next section"
      >
        <span className="text-[10px] font-semibold uppercase tracking-[0.2em]">
          Scroll
        </span>
        <ChevronDown className="h-5 w-5 animate-bounce-soft" />
      </a>

      <DemoModal />
    </section>
  );
}

function DemoModal() {
  const { demoModalOpen, setDemoModalOpen } = useUIStore();

  if (!demoModalOpen) return null;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="fixed inset-0 z-[60] flex items-center justify-center bg-void/80 p-4 backdrop-blur-md"
      onClick={() => setDemoModalOpen(false)}
    >
      <motion.div
        initial={{ scale: 0.95, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.25 }}
        className="glass-card w-full max-w-2xl overflow-hidden p-0"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-white/[0.06] px-5 py-4">
          <h3 className="font-semibold text-white">BugBee in 60 seconds</h3>
          <button
            type="button"
            onClick={() => setDemoModalOpen(false)}
            className="text-sm text-muted hover:text-white"
          >
            Close
          </button>
        </div>
        <div className="flex aspect-video items-center justify-center bg-surface-elevated">
          <div className="text-center">
            <Play className="mx-auto h-14 w-14 text-primary" />
            <p className="mt-3 text-sm text-muted">
              Demo video — connect a repo and watch bugs fall.
            </p>
            <Button href="#demo" className="mt-4" onClick={() => setDemoModalOpen(false)}>
              Try the live playground
            </Button>
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
}
