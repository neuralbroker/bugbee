import { Terminal } from "../components/Terminal";
import { SectionReveal } from "../components/SectionReveal";

export function InteractiveTerminal() {
  return <section id="terminal" className="terminal-section section-shell"><SectionReveal className="section-intro centered"><p className="kicker">INTERACTIVE WORKSPACE</p><h2>Try the terminal.</h2><p>Type a command. See the investigation as Bugbee sees it.</p></SectionReveal><SectionReveal delay={0.12}><Terminal commands={["bugbee hunt"]} /></SectionReveal></section>;
}
