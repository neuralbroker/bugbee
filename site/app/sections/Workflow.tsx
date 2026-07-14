"use client";

import { useLayoutEffect, useRef } from "react";
import { BrainCircuit, FileOutput, FolderGit2, Scale, ScanSearch, ShieldCheck, UserRoundCheck, Waypoints } from "lucide-react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { SectionReveal } from "../components/SectionReveal";

gsap.registerPlugin(ScrollTrigger);

const nodes = [
  [FolderGit2, "Repository", "Start with the work as it exists."], [Waypoints, "Local index", "Build usable code context."], [ScanSearch, "Security engines", "Search for meaningful risk."], [BrainCircuit, "AI review", "Add a model on your terms."], [Scale, "Risk score", "Prioritise what deserves attention."], [ShieldCheck, "Evidence", "Keep claims attached to proof."], [UserRoundCheck, "Human review", "Engineers decide the outcome."], [FileOutput, "SARIF", "Carry the result into the stack."],
] as const;

export function Workflow() {
  const ref = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => { if (!ref.current) return; const context = gsap.context(() => { gsap.fromTo(".workflow-pulse", { scaleY: 0, transformOrigin: "top" }, { scaleY: 1, ease: "none", scrollTrigger: { trigger: ref.current, start: "top 72%", end: "bottom 68%", scrub: true } }); }, ref); return () => context.revert(); }, []);
  return <section id="workflow" className="workflow-section section-shell"><SectionReveal className="section-intro centered"><p className="kicker">THE SECURITY LOOP</p><h2>From repository to remediation.</h2><p>A deliberate route from source code to a decision someone can stand behind.</p></SectionReveal><div ref={ref} className="workflow-track"><div className="workflow-rail"><i className="workflow-pulse" /></div>{nodes.map(([Icon, title, copy], index) => <SectionReveal key={title} delay={index * 0.03} direction={index % 2 ? "right" : "left"}><article className="workflow-node"><span className="workflow-index">{String(index + 1).padStart(2, "0")}</span><Icon aria-hidden="true" size={21} strokeWidth={1.5} /><div><h3>{title}</h3><p>{copy}</p></div></article></SectionReveal>)}</div></section>;
}
