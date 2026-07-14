"use client";

import type { MouseEvent, ReactNode } from "react";
import { motion, useMotionValue, useSpring } from "framer-motion";
import { cn } from "../lib/utils";

interface MagneticButtonProps {
  children: ReactNode;
  variant: "primary" | "secondary" | "ghost";
  href?: string;
  onClick?: () => void;
  className?: string;
  ariaLabel?: string;
}

export function MagneticButton({ children, variant, href, onClick, className, ariaLabel }: MagneticButtonProps) {
  const x = useSpring(useMotionValue(0), { stiffness: 150, damping: 15 });
  const y = useSpring(useMotionValue(0), { stiffness: 150, damping: 15 });
  const move = (event: MouseEvent<HTMLElement>) => {
    const box = event.currentTarget.getBoundingClientRect();
    x.set(((event.clientX - box.left) / box.width - 0.5) * 14);
    y.set(((event.clientY - box.top) / box.height - 0.5) * 14);
  };
  const reset = () => { x.set(0); y.set(0); };
  const classes = cn("magnetic-button", `magnetic-${variant}`, className);
  const content = <>{children}<span aria-hidden="true">↗</span></>;

  if (href) return <motion.a aria-label={ariaLabel} className={classes} href={href} style={{ x, y }} onMouseMove={move} onMouseLeave={reset}>{content}</motion.a>;
  return <motion.button aria-label={ariaLabel} className={classes} type="button" onClick={onClick} style={{ x, y }} onMouseMove={move} onMouseLeave={reset}>{content}</motion.button>;
}
