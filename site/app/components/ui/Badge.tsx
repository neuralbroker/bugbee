import { cn } from "@/app/lib/utils";

interface BadgeProps {
  children: React.ReactNode;
  className?: string;
  pulse?: boolean;
}

export function Badge({ children, className, pulse }: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 rounded-full border border-primary/40 bg-primary/10 px-3.5 py-1.5 text-xs font-medium text-primary",
        pulse && "animate-pulse-slow",
        className
      )}
    >
      {children}
    </span>
  );
}
