"use client";

import { useLenis } from "@/app/hooks/useLenis";

export function Providers({ children }: { children: React.ReactNode }) {
  useLenis();
  return <>{children}</>;
}
