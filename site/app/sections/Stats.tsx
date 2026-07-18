"use client";

import dynamic from "next/dynamic";
import { FadeIn } from "@/app/components/animations/FadeIn";
import { CountUp } from "@/app/components/animations/CountUp";

const ParticleField = dynamic(
  () =>
    import("@/app/components/three/ParticleField").then((m) => m.ParticleField),
  { ssr: false }
);

const STATS = [
  { to: 12000, suffix: "+", label: "developers" },
  { to: 50, suffix: "M+", label: "lines analyzed" },
  { to: 2, suffix: "M+", label: "bugs caught" },
  { to: 99.7, suffix: "%", label: "accuracy", decimals: 1 },
];

export function Stats() {
  return (
    <section
      id="stats"
      className="relative overflow-hidden border-y border-white/[0.06] bg-surface py-20 md:py-24"
    >
      <ParticleField />
      <div className="container-x relative z-10">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4 md:gap-6">
          {STATS.map((s, i) => (
            <FadeIn key={s.label} delay={0.08 * i} className="text-center">
              <p className="font-display text-3xl font-bold tracking-tight text-white sm:text-4xl lg:text-5xl">
                <CountUp
                  to={s.to}
                  suffix={s.suffix}
                  decimals={s.decimals ?? 0}
                />
              </p>
              <p className="mt-2 text-sm capitalize text-muted">{s.label}</p>
            </FadeIn>
          ))}
        </div>
      </div>
    </section>
  );
}
