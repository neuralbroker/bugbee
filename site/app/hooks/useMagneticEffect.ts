"use client";

import { useRef } from "react";
import {
  useMotionValue,
  useSpring,
  type MotionValue,
} from "framer-motion";

const SPRING = { damping: 15, stiffness: 150, mass: 0.1 };

export function useMagneticEffect(radius = 60) {
  const ref = useRef<HTMLElement>(null);
  const x = useMotionValue(0);
  const y = useMotionValue(0);
  const springX = useSpring(x, SPRING);
  const springY = useSpring(y, SPRING);

  const handleMouseMove = (e: React.MouseEvent) => {
    const el = ref.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height / 2;
    const dx = e.clientX - cx;
    const dy = e.clientY - cy;
    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist < radius) {
      x.set(dx * 0.3);
      y.set(dy * 0.3);
    }
  };

  const handleMouseLeave = () => {
    x.set(0);
    y.set(0);
  };

  return {
    ref,
    style: { x: springX as MotionValue<number>, y: springY as MotionValue<number> },
    handleMouseMove,
    handleMouseLeave,
  };
}
