"use client";

import type { ReactNode } from "react";
import { useLayoutEffect, useRef } from "react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { cn } from "../lib/utils";
import { useReducedMotion } from "../hooks/useReducedMotion";

gsap.registerPlugin(ScrollTrigger);

interface SectionRevealProps {
  children: ReactNode;
  delay?: number;
  direction?: "up" | "left" | "right";
  className?: string;
}

export function SectionReveal({ children, delay = 0, direction = "up", className }: SectionRevealProps) {
  const ref = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();
  useLayoutEffect(() => {
    if (reduced || !ref.current) return;
    const axis = direction === "up" ? { y: 52 } : { x: direction === "left" ? -52 : 52 };
    const context = gsap.context(() => {
      gsap.fromTo(ref.current, { opacity: 0, ...axis }, { opacity: 1, x: 0, y: 0, duration: 1, delay, ease: "power3.out", scrollTrigger: { trigger: ref.current, start: "top 84%", once: true } });
    }, ref);
    return () => context.revert();
  }, [delay, direction, reduced]);
  return <div ref={ref} className={cn("section-reveal", className)}>{children}</div>;
}
