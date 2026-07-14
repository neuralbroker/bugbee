import { BadgeCheck, UsersRound } from "lucide-react";
import { GlassCard } from "../components/GlassCard";
import { SectionReveal } from "../components/SectionReveal";

export function Enterprise() {
  return <section className="enterprise-section section-shell"><SectionReveal className="section-intro centered"><p className="kicker">FOR ENGINEERING TEAMS</p><h2>Scale with confidence.</h2></SectionReveal><div className="enterprise-cards"><SectionReveal direction="left"><GlassCard hover className="enterprise-card"><UsersRound size={28} strokeWidth={1.3}/><h3>Team workflows</h3><p>Human review, clear assignment, and approval chains that respect how security work actually gets done.</p></GlassCard></SectionReveal><SectionReveal delay={0.1} direction="right"><GlassCard hover className="enterprise-card"><BadgeCheck size={28} strokeWidth={1.3}/><h3>Compliance ready</h3><p>SARIF export, audit-ready evidence, and policy gates that make the record as useful as the result.</p></GlassCard></SectionReveal></div></section>;
}
