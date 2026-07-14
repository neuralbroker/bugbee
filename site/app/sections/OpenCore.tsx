import { Eye, Fingerprint } from "lucide-react";
import { MagneticButton } from "../components/MagneticButton";
import { SectionReveal } from "../components/SectionReveal";

export function OpenCore() {
  return <section className="core-section section-shell"><div className="core-grid" aria-hidden="true"/><SectionReveal className="core-content"><p className="kicker">TRANSPARENT BY DESIGN</p><h2>Know what is<br /><span>doing the work.</span></h2><p>Security tooling earns trust through inspection. Bugbee keeps the workflow legible, the evidence visible, and the engineer in control.</p><div className="core-signals"><span><Eye size={16}/> Inspectable workflow</span><span><Fingerprint size={16}/> Evidence-bound output</span></div><MagneticButton href="#access" variant="ghost">Request beta access</MagneticButton></SectionReveal></section>;
}
