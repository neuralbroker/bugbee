import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./app/**/*.{ts,tsx}"],
  theme: { extend: { fontFamily: { sans: ["var(--font-geist)", "sans-serif"] } } },
  plugins: [],
};

export default config;
