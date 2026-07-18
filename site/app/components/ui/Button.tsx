"use client";

import { forwardRef } from "react";
import { motion } from "framer-motion";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/app/lib/utils";
import { useMagneticEffect } from "@/app/hooks/useMagneticEffect";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-full font-semibold tracking-tight transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        primary:
          "bg-primary text-void shadow-glow-sm hover:bg-amber-400 hover:shadow-glow",
        secondary:
          "border border-white/15 bg-white/5 text-white backdrop-blur-sm hover:bg-white/10",
        ghost:
          "border border-white/10 bg-transparent text-white/90 hover:border-white/25 hover:bg-white/5",
        outline:
          "border border-primary/40 text-primary hover:bg-primary/10",
      },
      size: {
        sm: "h-9 px-4 text-sm",
        md: "h-11 px-6 text-sm",
        lg: "h-12 px-8 text-base",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "md",
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: VariantProps<typeof buttonVariants>["variant"];
  size?: VariantProps<typeof buttonVariants>["size"];
  magnetic?: boolean;
  href?: string;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      className,
      variant,
      size,
      magnetic = false,
      children,
      href,
      type = "button",
      onClick,
      disabled,
      ...props
    },
    forwardedRef
  ) => {
    const classes = cn(buttonVariants({ variant, size }), className);
    const mag = useMagneticEffect(60);

    if (magnetic) {
      const { ref, style, handleMouseMove, handleMouseLeave } = mag;
      const setRefs = (node: HTMLElement | null) => {
        (ref as React.MutableRefObject<HTMLElement | null>).current = node;
        if (typeof forwardedRef === "function") {
          forwardedRef(node as HTMLButtonElement);
        } else if (forwardedRef) {
          (forwardedRef as React.MutableRefObject<HTMLButtonElement | null>).current =
            node as HTMLButtonElement;
        }
      };

      if (href) {
        return (
          <motion.a
            ref={setRefs as React.Ref<HTMLAnchorElement>}
            href={href}
            className={classes}
            style={{ x: style.x, y: style.y }}
            onMouseMove={handleMouseMove}
            onMouseLeave={handleMouseLeave}
            onClick={onClick as unknown as React.MouseEventHandler<HTMLAnchorElement>}
          >
            {children}
          </motion.a>
        );
      }

      return (
        <motion.button
          ref={setRefs as React.Ref<HTMLButtonElement>}
          type={type}
          disabled={disabled}
          className={classes}
          style={{ x: style.x, y: style.y }}
          onMouseMove={handleMouseMove}
          onMouseLeave={handleMouseLeave}
          onClick={onClick}
        >
          {children}
        </motion.button>
      );
    }

    if (href) {
      return (
        <a
          href={href}
          className={classes}
          onClick={onClick as unknown as React.MouseEventHandler<HTMLAnchorElement>}
        >
          {children}
        </a>
      );
    }

    return (
      <button
        ref={forwardedRef}
        type={type}
        disabled={disabled}
        className={classes}
        onClick={onClick}
        {...props}
      >
        {children}
      </button>
    );
  }
);

Button.displayName = "Button";
