"use client";

import { useEffect, useState } from "react";
import { useReducedMotion } from "../hooks/useReducedMotion";

export function CursorGlow() {
  const [position, setPosition] = useState({ x: -300, y: -300 });
  const reduced = useReducedMotion();
  useEffect(() => {
    if (reduced) return;
    const move = (event: PointerEvent) => setPosition({ x: event.clientX, y: event.clientY });
    window.addEventListener("pointermove", move, { passive: true });
    return () => window.removeEventListener("pointermove", move);
  }, [reduced]);
  if (reduced) return null;
  return <div className="cursor-glow" aria-hidden="true" style={{ transform: `translate(${position.x - 120}px, ${position.y - 120}px)` }} />;
}
