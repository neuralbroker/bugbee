"use client";

import { useEffect, useState } from "react";
import { ArrowUp, Github, Linkedin, MessageCircle } from "lucide-react";
import { Logo } from "@/app/components/icons/Logo";
import { Input } from "@/app/components/ui/Input";
import { Button } from "@/app/components/ui/Button";

const COLUMNS = [
  {
    title: "Product",
    links: [
      { label: "Features", href: "#features" },
      { label: "How it works", href: "#how-it-works" },
      { label: "Pricing", href: "#pricing" },
      { label: "Live demo", href: "#demo" },
      { label: "Changelog", href: "#stats" },
    ],
  },
  {
    title: "Company",
    links: [
      { label: "About NeuralBroker", href: "https://github.com/neuralbroker" },
      { label: "GitHub", href: "https://github.com/neuralbroker/bugbee" },
      { label: "Security", href: "https://github.com/neuralbroker/bugbee/blob/main/SECURITY.md" },
      { label: "Blog", href: "#testimonials" },
    ],
  },
  {
    title: "Legal",
    links: [
      { label: "Privacy", href: "#" },
      { label: "Terms", href: "#" },
      { label: "License", href: "https://github.com/neuralbroker/bugbee/blob/main/LICENSE" },
    ],
  },
];

function XIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} fill="currentColor" aria-hidden>
      <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-4.714-6.231-5.401 6.231H2.744l7.727-8.835L1.254 2.25H8.08l4.253 5.622L18.244 2.25zm-1.161 17.52h1.833L7.084 4.126H5.117L17.083 19.77z" />
    </svg>
  );
}

const SOCIAL = [
  {
    label: "GitHub",
    href: "https://github.com/neuralbroker/bugbee",
    icon: Github,
  },
  {
    label: "X / Twitter",
    href: "https://x.com",
    icon: XIcon,
  },
  {
    label: "LinkedIn",
    href: "https://linkedin.com",
    icon: Linkedin,
  },
  {
    label: "Discord",
    href: "https://discord.com",
    icon: MessageCircle,
  },
];

export function Footer() {
  const [showTop, setShowTop] = useState(false);
  const [email, setEmail] = useState("");
  const [subscribed, setSubscribed] = useState(false);

  useEffect(() => {
    const onScroll = () => setShowTop(window.scrollY > 600);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <footer className="border-t border-white/[0.06] bg-void pb-10 pt-16">
      <div className="container-x">
        <div className="grid gap-10 md:grid-cols-2 lg:grid-cols-5">
          <div className="lg:col-span-2">
            <Logo size={36} />
            <p className="mt-4 max-w-xs text-sm leading-relaxed text-muted">
              AI-powered bug detection from NeuralBroker. Catch vulns before
              they bite — local-first, defensive-only.
            </p>
            <div className="mt-6 flex gap-2">
              {SOCIAL.map(({ label, href, icon: Icon }) => (
                <a
                  key={label}
                  href={href}
                  target="_blank"
                  rel="noopener noreferrer"
                  aria-label={label}
                  className="flex h-10 w-10 items-center justify-center rounded-full border border-white/10 bg-white/5 text-muted transition-all hover:-translate-y-1 hover:border-primary/40 hover:text-primary hover:shadow-glow-sm"
                >
                  <Icon className="h-4 w-4" />
                </a>
              ))}
            </div>
          </div>

          {COLUMNS.map((col) => (
            <div key={col.title}>
              <h4 className="text-sm font-semibold text-white">{col.title}</h4>
              <ul className="mt-4 space-y-2.5">
                {col.links.map((link) => (
                  <li key={link.label}>
                    <a
                      href={link.href}
                      className="text-sm text-muted transition-colors hover:text-white"
                    >
                      {link.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Newsletter */}
        <div className="mt-12 flex flex-col items-start justify-between gap-4 rounded-2xl border border-white/[0.06] bg-white/[0.02] p-6 sm:flex-row sm:items-center">
          <div>
            <p className="font-semibold text-white">Stay in the loop</p>
            <p className="text-sm text-muted">
              Release notes, security tips, no spam.
            </p>
          </div>
          {subscribed ? (
            <p className="text-sm text-success">Subscribed. Welcome aboard.</p>
          ) : (
            <form
              className="flex w-full max-w-sm gap-2"
              onSubmit={(e) => {
                e.preventDefault();
                if (email.includes("@")) setSubscribed(true);
              }}
            >
              <Input
                type="email"
                placeholder="Email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
              />
              <Button type="submit" size="md">
                Subscribe
              </Button>
            </form>
          )}
        </div>

        <div className="mt-10 flex flex-col items-center justify-between gap-3 border-t border-white/[0.06] pt-8 text-sm text-muted sm:flex-row">
          <p>© 2026 NeuralBroker. Built with 🐝 for developers.</p>
          <p className="text-xs">
            Defensive security tooling only. No exploitation.
          </p>
        </div>
      </div>

      {showTop && (
        <button
          type="button"
          onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
          className="fixed bottom-6 right-6 z-40 flex h-11 w-11 items-center justify-center rounded-full border border-primary/30 bg-primary text-void shadow-glow transition-transform hover:scale-105"
          aria-label="Back to top"
        >
          <ArrowUp className="h-5 w-5" />
        </button>
      )}
    </footer>
  );
}
