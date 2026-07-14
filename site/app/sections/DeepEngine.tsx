import { BrainCircuit, Fingerprint, Network } from "lucide-react";
import { GlassCard } from "../components/GlassCard";
import { SectionReveal } from "../components/SectionReveal";

const points = [[Network, "Repository indexing", "Full graph analysis, not a longer grep."], [BrainCircuit, "AI-assisted hunting", "A model you choose, informed by the surrounding code."], [Fingerprint, "Evidence correlation", "Findings connected to locations, traces, and decisions."]] as const;

export function DeepEngine() {
  return <section className="deep-section section-shell"><SectionReveal className="section-intro"><p className="kicker">THE DEEP ENGINE</p><h2>Deep analysis.<br /><span>No noise.</span></h2></SectionReveal><div className="deep-cards">{points.map(([Icon, title, copy], index) => <SectionReveal key={title} delay={index * 0.1}><GlassCard hover className={`deep-card deep-card-${index + 1}`}><Icon size={23} strokeWidth={1.4} /><h3>{title}</h3><p>{copy}</p></GlassCard></SectionReveal>)}</div></section>;
}
