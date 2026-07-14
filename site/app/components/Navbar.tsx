"use client";

import { Menu, X } from "lucide-react";
import { useEffect, useState } from "react";

export function Navbar() {
  const [open, setOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => { const update = () => setScrolled(window.scrollY > 18); update(); window.addEventListener("scroll", update, { passive: true }); return () => window.removeEventListener("scroll", update); }, []);
  const links = [["Product", "#product"], ["Workflow", "#workflow"], ["Architecture", "#architecture"]] as const;
  return <nav className={`navbar ${scrolled ? "navbar-scrolled" : ""}`} aria-label="Main navigation"><a className="wordmark" href="#top">Bugbee<span>.</span></a><div className="nav-links">{links.map(([label, href]) => <a key={href} href={href}>{label}</a>)}<a className="nav-cta" href="#access">Join beta</a></div><button className="menu-button" type="button" aria-label={open ? "Close menu" : "Open menu"} aria-expanded={open} onClick={() => setOpen(!open)}>{open ? <X /> : <Menu />}</button>{open && <div className="mobile-menu">{links.map(([label, href]) => <a key={href} href={href} onClick={() => setOpen(false)}>{label}</a>)}<a href="#access" onClick={() => setOpen(false)}>Join private beta</a></div>}</nav>;
}
