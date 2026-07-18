import { create } from "zustand";

interface UIState {
  mobileMenuOpen: boolean;
  theme: "dark" | "light";
  demoModalOpen: boolean;
  setMobileMenuOpen: (open: boolean) => void;
  toggleMobileMenu: () => void;
  toggleTheme: () => void;
  setDemoModalOpen: (open: boolean) => void;
}

export const useUIStore = create<UIState>((set) => ({
  mobileMenuOpen: false,
  theme: "dark",
  demoModalOpen: false,
  setMobileMenuOpen: (open) => set({ mobileMenuOpen: open }),
  toggleMobileMenu: () => set((s) => ({ mobileMenuOpen: !s.mobileMenuOpen })),
  toggleTheme: () =>
    set((s) => ({ theme: s.theme === "dark" ? "light" : "dark" })),
  setDemoModalOpen: (open) => set({ demoModalOpen: open }),
}));
