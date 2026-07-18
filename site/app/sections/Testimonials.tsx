"use client";

import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Star, Quote } from "lucide-react";
import { FadeIn } from "@/app/components/animations/FadeIn";
import { Card } from "@/app/components/ui/Card";
import type { Testimonial } from "@/app/types";

const TESTIMONIALS: Testimonial[] = [
  {
    quote:
      "BugBee caught a SQL injection in a PR that three humans reviewed. We merged the fix before lunch. That's the only review bot I'll defend in Slack.",
    name: "Maya Chen",
    title: "Staff Security Engineer",
    company: "Lattice",
    rating: 5,
    initials: "MC",
  },
  {
    quote:
      "We swapped a noisy SAST tool for BugBee. False positives dropped, real vulns went up. The AI patches are actually reviewable — not black-box magic.",
    name: "Jordan Blake",
    title: "VP Engineering",
    company: "Northwind",
    rating: 5,
    initials: "JB",
  },
  {
    quote:
      "Local-first + BYOK was the deal. Our compliance team loves that source never has to leave the VPC. I love that it runs in CI without babysitting.",
    name: "Priya Nair",
    title: "Platform Lead",
    company: "Helios Health",
    rating: 5,
    initials: "PN",
  },
];

function Stars({ n }: { n: number }) {
  return (
    <div className="flex gap-0.5">
      {Array.from({ length: n }).map((_, i) => (
        <Star key={i} className="h-3.5 w-3.5 fill-primary text-primary" />
      ))}
    </div>
  );
}

function TestimonialCard({ t }: { t: Testimonial }) {
  return (
    <Card className="relative h-full overflow-hidden">
      <Quote className="absolute right-4 top-4 h-12 w-12 text-white/[0.04]" />
      <Stars n={t.rating} />
      <p className="mt-4 text-[15px] leading-relaxed text-white/90">
        &ldquo;{t.quote}&rdquo;
      </p>
      <div className="mt-6 flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-full bg-gradient-to-br from-primary/40 to-secondary/40 text-sm font-bold text-white">
          {t.initials}
        </div>
        <div>
          <p className="text-sm font-semibold text-white">{t.name}</p>
          <p className="text-xs text-muted">
            {t.title} · {t.company}
          </p>
        </div>
      </div>
    </Card>
  );
}

export function Testimonials() {
  const [index, setIndex] = useState(0);

  useEffect(() => {
    const id = setInterval(
      () => setIndex((i) => (i + 1) % TESTIMONIALS.length),
      5000
    );
    return () => clearInterval(id);
  }, []);

  return (
    <section id="testimonials" className="section-pad relative">
      <div className="container-x">
        <FadeIn className="mx-auto max-w-2xl text-center">
          <p className="mb-4 text-xs font-semibold uppercase tracking-[0.2em] text-primary">
            Testimonials
          </p>
          <h2 className="font-display text-[clamp(2rem,4.5vw,3.5rem)] font-semibold leading-[1.1] tracking-[-0.02em] text-white">
            Loved by developers who ship.
          </h2>
        </FadeIn>

        {/* Desktop grid */}
        <div className="mt-14 hidden gap-6 md:grid md:grid-cols-3">
          {TESTIMONIALS.map((t, i) => (
            <FadeIn key={t.name} delay={0.08 * i}>
              <TestimonialCard t={t} />
            </FadeIn>
          ))}
        </div>

        {/* Mobile carousel */}
        <div className="mt-10 md:hidden">
          <AnimatePresence mode="wait">
            <motion.div
              key={index}
              initial={{ opacity: 0, x: 24 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -24 }}
              transition={{ duration: 0.3 }}
            >
              <TestimonialCard t={TESTIMONIALS[index]} />
            </motion.div>
          </AnimatePresence>
          <div className="mt-4 flex justify-center gap-2">
            {TESTIMONIALS.map((_, i) => (
              <button
                key={i}
                type="button"
                aria-label={`Go to testimonial ${i + 1}`}
                onClick={() => setIndex(i)}
                className={`h-1.5 rounded-full transition-all ${
                  i === index ? "w-6 bg-primary" : "w-1.5 bg-white/20"
                }`}
              />
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
