"use client";

import dynamic from "next/dynamic";
import Image from "next/image";
import { ArrowDown } from "lucide-react";
import { Terminal } from "../components/Terminal";
import { MagneticButton } from "../components/MagneticButton";

const FloatingObject = dynamic(() => import("../components/FloatingObject").then((module) => module.FloatingObject), { ssr: false, loading: () => null });

export function Hero() {
  return <section id="top" className="hero-section"><FloatingObject /><Image className="hero-art" src="/bugbee-og-v2.png" alt="Abstract terminal and signal composition" width={1731} height={909} priority sizes="100vw" /><div className="hero-vignette" /><div className="hero-content"><p className="kicker">PRIVATE BETA · SECURITY INFRASTRUCTURE</p><h1>Terminal-first<br /><span>AI Security IDE</span></h1><p className="hero-copy">Bugbee is where elite engineers secure software.</p><div className="hero-actions"><MagneticButton href="#access" variant="primary">Join private beta</MagneticButton><MagneticButton href="#terminal" variant="secondary">Explore the terminal</MagneticButton></div></div><div className="hero-terminal"><Terminal interactive={false} height="auto" compact /></div><a className="scroll-cue" href="#product" aria-label="Scroll to product overview"><span>SCROLL TO EXPLORE</span><ArrowDown size={15} /></a></section>;
}
