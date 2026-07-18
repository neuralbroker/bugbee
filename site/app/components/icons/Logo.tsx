import { cn } from "@/app/lib/utils";

interface LogoProps {
  className?: string;
  showWordmark?: boolean;
  size?: number;
}

export function Logo({ className, showWordmark = true, size = 32 }: LogoProps) {
  return (
    <a href="#top" className={cn("inline-flex items-center gap-2.5", className)}>
      <svg
        width={size}
        height={size}
        viewBox="0 0 40 40"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-hidden
      >
        <rect width="40" height="40" rx="10" fill="url(#bee-bg)" />
        <path
          d="M20 8c-1.2 0-2.2.7-2.6 1.7L14 18.5c-.3.8.1 1.7.9 2.1l2.6 1.3v2.6c0 .9.7 1.6 1.6 1.6h1.8c.9 0 1.6-.7 1.6-1.6v-2.6l2.6-1.3c.8-.4 1.2-1.3.9-2.1l-3.4-8.8C22.2 8.7 21.2 8 20 8z"
          fill="#f59e0b"
        />
        <ellipse cx="17.2" cy="12.5" rx="2.2" ry="3.5" fill="#fbbf24" opacity="0.9" transform="rotate(-25 17.2 12.5)" />
        <ellipse cx="22.8" cy="12.5" rx="2.2" ry="3.5" fill="#fbbf24" opacity="0.9" transform="rotate(25 22.8 12.5)" />
        <circle cx="18.2" cy="15.2" r="0.9" fill="#050508" />
        <circle cx="21.8" cy="15.2" r="0.9" fill="#050508" />
        <path
          d="M16 22.5c1.2 1.5 2.5 2.2 4 2.2s2.8-.7 4-2.2"
          stroke="#8b5cf6"
          strokeWidth="1.2"
          strokeLinecap="round"
          fill="none"
        />
        <path d="M12 28h16" stroke="#f59e0b" strokeWidth="1.5" strokeLinecap="round" opacity="0.5" />
        <path d="M14 31h12" stroke="#22d3ee" strokeWidth="1" strokeLinecap="round" opacity="0.35" />
        <defs>
          <linearGradient id="bee-bg" x1="0" y1="0" x2="40" y2="40">
            <stop stopColor="#1a1520" />
            <stop offset="1" stopColor="#0a0a10" />
          </linearGradient>
        </defs>
      </svg>
      {showWordmark && (
        <span className="font-display text-lg font-bold tracking-tight text-white">
          Bug<span className="text-primary">Bee</span>
        </span>
      )}
    </a>
  );
}
