import { forwardRef } from "react";
import { cn } from "@/app/lib/utils";

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, type = "text", ...props }, ref) => {
    return (
      <input
        type={type}
        ref={ref}
        className={cn(
          "h-12 w-full rounded-full border border-white/10 bg-white/5 px-5 text-sm text-white placeholder:text-muted outline-none transition-colors focus:border-primary/50 focus:bg-white/[0.07]",
          className
        )}
        {...props}
      />
    );
  }
);

Input.displayName = "Input";
