import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./app/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        void: "#050508",
        surface: {
          DEFAULT: "#0a0a10",
          elevated: "#111118",
        },
        primary: {
          DEFAULT: "#f59e0b",
          glow: "rgba(245, 158, 11, 0.15)",
        },
        secondary: "#8b5cf6",
        accent: "#22d3ee",
        muted: "#94a3b8",
        success: "#22c55e",
        error: "#ef4444",
      },
      fontFamily: {
        sans: ["var(--font-space)", "var(--font-geist)", "system-ui", "sans-serif"],
        mono: ["var(--font-mono)", "ui-monospace", "monospace"],
        display: ["var(--font-space)", "system-ui", "sans-serif"],
      },
      maxWidth: {
        container: "1280px",
      },
      boxShadow: {
        glow: "0 0 40px rgba(245, 158, 11, 0.25)",
        "glow-sm": "0 0 20px rgba(245, 158, 11, 0.15)",
        "glow-purple": "0 0 40px rgba(139, 92, 246, 0.25)",
        card: "0 20px 60px rgba(0, 0, 0, 0.4)",
      },
      backgroundImage: {
        "gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
        noise:
          "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.05'/%3E%3C/svg%3E\")",
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        float: "float 6s ease-in-out infinite",
        "spin-slow": "spin 8s linear infinite",
        "border-spin": "border-spin 4s linear infinite",
        blink: "blink 1s step-end infinite",
        "bounce-soft": "bounce-soft 2s ease-in-out infinite",
      },
      keyframes: {
        float: {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-12px)" },
        },
        "border-spin": {
          "0%": { transform: "rotate(0deg)" },
          "100%": { transform: "rotate(360deg)" },
        },
        blink: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0" },
        },
        "bounce-soft": {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(8px)" },
        },
      },
    },
  },
  plugins: [],
};

export default config;
