"use client";

import { FadeIn } from "@/app/components/animations/FadeIn";
import { Accordion } from "@/app/components/ui/Accordion";
import type { FAQItem } from "@/app/types";

const ITEMS: FAQItem[] = [
  {
    question: "What languages does BugBee support?",
    answer:
      "JavaScript/TypeScript, Python, Go, PHP, Java, and Rust for core engines. More languages ship via rule packs. Deterministic scanners cover secrets, taint, and OWASP patterns; the AI layer reasons over any text file you point it at.",
  },
  {
    question: "How does CI/CD integration work?",
    answer:
      "Install the GitHub/GitLab/Bitbucket app or run the CLI in your pipeline. BugBee posts status checks, annotates PRs with findings, and can block merges on critical severity. SARIF export works with any toolchain that speaks SARIF.",
  },
  {
    question: "Is my code secure?",
    answer:
      "Yes. BugBee is local-first and defensive-only — no live exploitation. Secrets are redacted before any model call. Enterprise plans support fully air-gapped and VPC deployments with BYOK models so source never leaves your perimeter.",
  },
  {
    question: "Can I customize rules?",
    answer:
      "Absolutely. Encode org standards as custom rule packs, tune severity, and whitelist known-safe patterns. Pro and Enterprise include custom rules; Enterprise can train AI on your internal patterns.",
  },
  {
    question: "What's the Free vs Pro difference?",
    answer:
      "Free is for individuals: 1 repo, 100 scans/month, core packs. Pro unlocks unlimited repos, AI fix suggestions, CI/CD gates, team dashboard, custom rules, and priority support — $29/dev/month (20% off annual).",
  },
  {
    question: "How do I get started?",
    answer:
      "Drop your email below, or curl our install script and run `bugbee init` in your repo. Connect GitHub in one click, or stay fully local with the TUI. No credit card for Free.",
  },
];

export function FAQ() {
  return (
    <section id="faq" className="section-pad relative">
      <div className="container-x">
        <FadeIn className="mx-auto max-w-2xl text-center">
          <p className="mb-4 text-xs font-semibold uppercase tracking-[0.2em] text-primary">
            FAQ
          </p>
          <h2 className="font-display text-[clamp(2rem,4.5vw,3.5rem)] font-semibold leading-[1.1] tracking-[-0.02em] text-white">
            Questions? We&apos;ve got answers.
          </h2>
        </FadeIn>

        <FadeIn delay={0.1} className="mx-auto mt-12 max-w-2xl">
          <Accordion items={ITEMS} />
        </FadeIn>
      </div>
    </section>
  );
}
