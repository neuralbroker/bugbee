"use client";

import type { ReactNode } from "react";
import { useLenis } from "../hooks/useLenis";
import { CursorGlow } from "./CursorGlow";
import { LoadingScreen } from "./LoadingScreen";
import { Navbar } from "./Navbar";

export function AppShell({ children }: { children: ReactNode }) {
  useLenis();
  return <><LoadingScreen /><CursorGlow /><Navbar />{children}</>;
}
