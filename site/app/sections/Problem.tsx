"use client";

import { Bug, Clock, ShieldAlert } from "lucide-react";
import { FadeIn } from "@/app/components/animations/FadeIn";
import {
  StaggerContainer,
  StaggerItem,
} from "@/app/components/animations/StaggerContainer";
import { CountUp } from "@/app/components/animations/CountUp";
import { Card } from "@/app/components/ui/Card";

const CARDS = [
  {
    icon: Bug,
    stat: 70,
    suffix: "%",
    title: "of bugs ship to production",
    body: "You review the PR. CI is green. Users still find the edge case you never wrote a test for.",
    color: "text-error",
    pulse: "bg-error/20",
  },
  {
    icon: Clock,
    stat: 40,
    suffix: "%",
    title: "of senior dev time wasted on reviews",
    body: "Senior engineers aren't a lint rule. Free them from nits so they can ship architecture.",
    color: "text-primary",
    pulse: "bg-primary/20",
  },
  {
    icon: ShieldAlert,
    stat: 100,
    suffix: "x",
    title: "more expensive post-deploy",
    body: "Security fixes after ship cost orders of magnitude more. Catch vulns while the PR is still open.",
    color: "text-secondary",
    pulse: "bg-secondary/20",
  },
];

export function Problem() {
  return (
    <section id="problem" className="section-pad relative">
      <div className="container-x">
        <FadeIn className="mx-auto max-w-3xl text-center">
          <p className="mb-4 text-xs font-semibold uppercase tracking-[0.2em] text-primary">
            The Bug Epidemic
          </p>
          <h2 className="font-display text-[clamp(2rem,4.5vw,3.5rem)] font-semibold leading-[1.1] tracking-[-0.02em] text-white">
            Your code has bugs. You just don&apos;t know where.
          </h2>
          <p className="mt-5 text-base leading-relaxed text-muted sm:text-lg">
            Manual review doesn&apos;t scale. Static analysis alone is noisy.
            BugBee sits in the middle — smart enough to find real issues,
            humble enough to show its work.
          </p>
        </FadeIn>

        <StaggerContainer className="mt-14 grid gap-6 md:grid-cols-3">
          {CARDS.map((card) => (
            <StaggerItem key={card.title}>
              <Card className="h-full">
                <div
                  className={`mb-5 inline-flex h-12 w-12 items-center justify-center rounded-xl ${card.pulse}`}
                >
                  <card.icon className={`h-6 w-6 ${card.color}`} />
                </div>
                <p className="font-display text-4xl font-bold tracking-tight text-white sm:text-5xl">
                  <CountUp to={card.stat} suffix={card.suffix} />
                </p>
                <h3 className="mt-2 text-lg font-semibold text-white">
                  {card.title}
                </h3>
                <p className="mt-3 text-sm leading-relaxed text-muted">
                  {card.body}
                </p>
              </Card>
            </StaggerItem>
          ))}
        </StaggerContainer>
      </div>
    </section>
  );
}
