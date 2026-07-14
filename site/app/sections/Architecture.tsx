import { Braces, Cpu, FileCode2, FolderTree, ScanLine } from "lucide-react";
import { SectionReveal } from "../components/SectionReveal";

const stack = [[FolderTree, "Local filesystem"], [Cpu, "BYOK model"], [ScanLine, "Security engines"], [Braces, "SARIF output"]] as const;

export function Architecture() {
  return <section id="architecture" className="architecture-section section-shell"><SectionReveal className="section-intro"><p className="kicker">ARCHITECTURE</p><h2>Built for engineers.</h2><p>Bring your own model. Keep the source close. Export work that other tools can understand.</p></SectionReveal><SectionReveal delay={0.12}><div className="architecture-stack">{stack.map(([Icon, label], index) => <div className="architecture-node" key={label}><span>{String(index + 1).padStart(2, "0")}</span><Icon size={22} strokeWidth={1.4}/><b>{label}</b>{index < stack.length - 1 && <i aria-hidden="true" />}</div>)}<div className="architecture-signal" aria-hidden="true" /></div></SectionReveal><p className="architecture-note"><FileCode2 size={17} /> Local-first. Evidence-native. Git-integrated.</p></section>;
}
