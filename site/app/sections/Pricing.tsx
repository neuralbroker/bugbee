"use client";

import { useState } from "react";
import { Check, X } from "lucide-react";
import { FadeIn } from "@/app/components/animations/FadeIn";
import { Button } from "@/app/components/ui/Button";
import { cn } from "@/app/lib/utils";

const TIERS = [
  {
    id: "free",
    name: "Free",
    monthly: 0,
    description: "For solo builders shipping side projects.",
    features: [
      "1 private repo",
      "100 scans / month",
      "Core rule packs",
      "Community support",
    ],
    cta: "Start Free",
    popular: false,
  },
  {
    id: "pro",
    name: "Pro",
    monthly: 29,
    description: "For teams that merge fast and sleep well.",
    features: [
      "Unlimited repos",
      "AI fix suggestions",
      "CI/CD status checks",
      "Team dashboard",
      "Priority support",
      "Custom rules",
    ],
    cta: "Start Pro Trial",
    popular: true,
  },
  {
    id: "enterprise",
    name: "Enterprise",
    monthly: null as number | null,
    description: "For regulated orgs and large monorepos.",
    features: [
      "SSO / SAML",
      "On-prem & VPC deploy",
      "Dedicated support",
      "Custom AI training",
      "SLA + audit logs",
      "India AppSec packs",
    ],
    cta: "Contact Sales",
    popular: false,
  },
];

const COMPARISON = [
  { feature: "Private repos", free: "1", pro: "Unlimited", enterprise: "Unlimited" },
  { feature: "Scans / month", free: "100", pro: "Unlimited", enterprise: "Unlimited" },
  { feature: "AI fixes", free: false, pro: true, enterprise: true },
  { feature: "CI/CD integration", free: false, pro: true, enterprise: true },
  { feature: "Custom rules", free: false, pro: true, enterprise: true },
  { feature: "SSO", free: false, pro: false, enterprise: true },
  { feature: "On-prem", free: false, pro: false, enterprise: true },
  { feature: "Dedicated support", free: false, pro: false, enterprise: true },
];

export function Pricing() {
  const [annual, setAnnual] = useState(true);

  return (
    <section id="pricing" className="section-pad relative">
      <div className="container-x">
        <FadeIn className="mx-auto max-w-2xl text-center">
          <p className="mb-4 text-xs font-semibold uppercase tracking-[0.2em] text-primary">
            Pricing
          </p>
          <h2 className="font-display text-[clamp(2rem,4.5vw,3.5rem)] font-semibold leading-[1.1] tracking-[-0.02em] text-white">
            Simple pricing. Powerful results.
          </h2>
          <p className="mt-5 text-base text-muted sm:text-lg">
            Start free. Upgrade when the bugs stop shipping themselves.
          </p>

          <div className="mt-8 inline-flex items-center gap-1 rounded-full border border-white/10 bg-white/5 p-1">
            <button
              type="button"
              onClick={() => setAnnual(false)}
              className={cn(
                "rounded-full px-4 py-2 text-sm font-medium transition-colors",
                !annual ? "bg-primary text-void" : "text-muted hover:text-white"
              )}
            >
              Monthly
            </button>
            <button
              type="button"
              onClick={() => setAnnual(true)}
              className={cn(
                "rounded-full px-4 py-2 text-sm font-medium transition-colors",
                annual ? "bg-primary text-void" : "text-muted hover:text-white"
              )}
            >
              Annual{" "}
              <span className="ml-1 text-[10px] font-bold uppercase opacity-80">
                Save 20%
              </span>
            </button>
          </div>
        </FadeIn>

        <div className="mt-12 grid gap-6 lg:grid-cols-3">
          {TIERS.map((tier, i) => {
            const price =
              tier.monthly === null
                ? null
                : annual
                  ? Math.round(tier.monthly * 0.8)
                  : tier.monthly;

            return (
              <FadeIn key={tier.id} delay={0.08 * i}>
                <div
                  className={cn(
                    "glass-card relative flex h-full flex-col p-7",
                    tier.popular &&
                      "border-primary/40 shadow-glow ring-1 ring-primary/30"
                  )}
                >
                  {tier.popular && (
                    <span className="absolute -top-3 left-1/2 -translate-x-1/2 rounded-full bg-primary px-3 py-0.5 text-[10px] font-bold uppercase tracking-wider text-void">
                      Popular
                    </span>
                  )}
                  <h3 className="text-xl font-semibold text-white">
                    {tier.name}
                  </h3>
                  <p className="mt-2 text-sm text-muted">{tier.description}</p>
                  <div className="mt-6 flex items-baseline gap-1">
                    {price === null ? (
                      <span className="font-display text-4xl font-bold text-white">
                        Custom
                      </span>
                    ) : (
                      <>
                        <span className="font-display text-4xl font-bold text-white">
                          ${price}
                        </span>
                        <span className="text-sm text-muted">
                          {tier.id === "pro" ? "/dev/mo" : "/mo"}
                        </span>
                      </>
                    )}
                  </div>
                  <ul className="mt-6 flex-1 space-y-3">
                    {tier.features.map((f) => (
                      <li
                        key={f}
                        className="flex items-start gap-2 text-sm text-white/85"
                      >
                        <Check className="mt-0.5 h-4 w-4 shrink-0 text-primary" />
                        {f}
                      </li>
                    ))}
                  </ul>
                  <Button
                    href="#cta"
                    variant={tier.popular ? "primary" : "secondary"}
                    className="mt-8 w-full"
                    magnetic={tier.popular}
                  >
                    {tier.cta}
                  </Button>
                </div>
              </FadeIn>
            );
          })}
        </div>

        {/* Comparison table */}
        <FadeIn delay={0.2} className="mt-16 overflow-x-auto">
          <table className="w-full min-w-[560px] border-collapse text-left text-sm">
            <thead>
              <tr className="border-b border-white/10">
                <th className="py-3 pr-4 font-medium text-muted">Feature</th>
                <th className="px-4 py-3 font-medium text-white">Free</th>
                <th className="px-4 py-3 font-medium text-primary">Pro</th>
                <th className="px-4 py-3 font-medium text-white">Enterprise</th>
              </tr>
            </thead>
            <tbody>
              {COMPARISON.map((row) => (
                <tr
                  key={row.feature}
                  className="border-b border-white/[0.04]"
                >
                  <td className="py-3.5 pr-4 text-muted">{row.feature}</td>
                  {(["free", "pro", "enterprise"] as const).map((k) => {
                    const val = row[k];
                    return (
                      <td key={k} className="px-4 py-3.5 text-white/90">
                        {typeof val === "boolean" ? (
                          val ? (
                            <Check className="h-4 w-4 text-success" />
                          ) : (
                            <X className="h-4 w-4 text-white/20" />
                          )
                        ) : (
                          val
                        )}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </FadeIn>
      </div>
    </section>
  );
}
