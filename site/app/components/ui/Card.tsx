"use client";

import { forwardRef } from "react";
import { motion } from "framer-motion";
import { cn } from "@/app/lib/utils";

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  hover?: boolean;
  glow?: boolean;
}

export const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ className, hover = true, glow = false, children, ...props }, ref) => {
    return (
      <motion.div
        ref={ref}
        className={cn(
          "glass-card gradient-border p-6",
          glow && "shadow-glow-sm",
          className
        )}
        whileHover={
          hover
            ? {
                y: -8,
                boxShadow: "0 24px 60px rgba(0,0,0,0.45), 0 0 30px rgba(245,158,11,0.08)",
                transition: { duration: 0.25 },
              }
            : undefined
        }
        {...(props as React.ComponentPropsWithoutRef<typeof motion.div>)}
      >
        {children}
      </motion.div>
    );
  }
);

Card.displayName = "Card";
