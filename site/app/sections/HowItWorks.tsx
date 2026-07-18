"use client";

import { useRef } from "react";
import { motion, useScroll, useTransform } from "framer-motion";
import { Brain, Github, Wand2 } from "lucide-react";
import { FadeIn } from "@/app/components/animations/FadeIn";

const STEPS = [
  {
    num: "01",
    icon: Github,
    title: "Connect",
    body: "One-click repo integration. Install the GitHub App — or point BugBee at your local workspace. No cloud lock-in required.",
  },
  {
    num: "02",
    icon: Brain,
    title: "Analyze",
    body: "Our neural engine scans every line: taint paths, secrets, OWASP packs, and custom rules — on every commit.",
  },
  {
    num: "03",
    icon: Wand2,
    title: "Fix",
    body: "AI suggests patches with evidence. You approve. CI stays green. Users never see the bug that almost shipped.",
  },
];

export function HowItWorks() {
  const ref = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: ref,
    offset: ["start 0.8", "end 0.5"],
  });
  const pathLength = useTransform(scrollYProgress, [0, 1], [0, 1]);

  return (
    <section id="how-it-works" className="section-pad relative">
      <div className="container-x">
        <FadeIn className="mx-auto max-w-2xl text-center">
          <p className="mb-4 text-xs font-semibold uppercase tracking-[0.2em] text-primary">
            How it works
          </p>
          <h2 className="font-display text-[clamp(2rem,4.5vw,3.5rem)] font-semibold leading-[1.1] tracking-[-0.02em] text-white">
            Three steps to bug-free code.
          </h2>
        </FadeIn>

        <div ref={ref} className="relative mt-16">
          {/* Desktop connecting line */}
          <div className="pointer-events-none absolute left-0 right-0 top-10 hidden h-0.5 md:block">
            <svg
              className="h-2 w-full overflow-visible"
              viewBox="0 0 100 2"
              preserveAspectRatio="none"
              aria-hidden
            >
              <path
                d="M10 1 H90"
                fill="none"
                stroke="rgba(255,255,255,0.08)"
                strokeWidth="0.5"
              />
              <motion.path
                d="M10 1 H90"
                fill="none"
                stroke="#f59e0b"
                strokeWidth="0.5"
                strokeLinecap="round"
                style={{ pathLength }}
              />
            </svg>
          </div>

          <div className="grid gap-8 md:grid-cols-3 md:gap-6">
            {STEPS.map((step, i) => (
              <FadeIn key={step.num} delay={0.1 * i}>
                <div className="relative flex flex-col items-center text-center md:items-start md:text-left">
                  <div className="relative z-10 mb-6 flex h-20 w-20 items-center justify-center rounded-2xl border border-primary/30 bg-surface shadow-glow-sm">
                    <span className="absolute -top-2 -right-2 flex h-7 w-7 items-center justify-center rounded-full bg-primary font-mono text-[10px] font-bold text-void shadow-glow-sm">
                      {step.num}
                    </span>
                    <step.icon className="h-8 w-8 text-primary" />
                  </div>
                  <h3 className="text-xl font-semibold text-white">
                    {step.title}
                  </h3>
                  <p className="mt-3 text-sm leading-relaxed text-muted">
                    {step.body}
                  </p>
                </div>
              </FadeIn>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
