import { SectionReveal } from "../components/SectionReveal";

export function WhyBugbee() {
  return <section id="product" className="why-section section-shell"><SectionReveal><p className="kicker">WHY BUGBEE</p><h2>Not another<br /><span>AI scanner.</span></h2></SectionReveal><SectionReveal delay={0.16} direction="right" className="why-copy"><p>Bugbee is an IDE. Your code stays local. Your model is yours. We don&apos;t upload repositories to sell a dashboard. We build a considered workspace for engineers who ship.</p><div className="principle-list"><span>LOCAL FIRST</span><span>BYOK MODELS</span><span>HUMAN DECISIONS</span></div></SectionReveal><div className="why-orbit" aria-hidden="true"><i/><i/><b/></div></section>;
}
