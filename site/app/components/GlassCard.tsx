import type { ReactNode } from "react";
import { cn } from "../lib/utils";

interface GlassCardProps {
  children: ReactNode;
  className?: string;
  hover?: boolean;
}

export function GlassCard({ children, className, hover = false }: GlassCardProps) {
  return <div className={cn("glass-card", hover && "glass-card-hover", className)}>{children}</div>;
}
