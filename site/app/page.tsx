"use client";

import { PointerEvent, useEffect, useState } from "react";

const features = [
  ["01", "Local first", "Analysis begins in your environment. Model review is optional and remains under your control."],
  ["02", "Evidence attached", "Locations, traces, risk scores, and review state stay connected to each finding."],
  ["03", "Human review", "Bugbee can narrow the work. Engineers still make the security decision."],
  ["04", "Built for the terminal", "A focused workflow for investigation, review, and SARIF export without a browser-bound dashboard."],
];

const workflow = [
  ["Repository", "Where the work lives"],
  ["Local analysis", "Build context"],
  ["Security engines", "Find the signal"],
  ["Optional review", "Add a second look"],
  ["Human decision", "Make the call"],
  ["Report", "Carry context forward"],
];

const terminalCommands = {
  "bugbee hunt": ["Indexing repository...", "✓ Local index ready                 218 files", "✓ Rules and secrets checks complete", "✓ Taint analysis complete", "✓ Review queue prepared", "", "Findings ready for review              illustrative"],
  "bugbee findings": ["Loading local finding store...", "42 findings across 218 files", "12 high · 18 medium · 12 low", "Evidence completeness                  98%", "", "Use bugbee review <id> to inspect one"],
  "bugbee review": ["Opening human review queue...", "a8f3c1e2  HIGH     needs review", "f1029d7b  MEDIUM   needs review", "c71a082e  LOW      confirmed", "", "Your decision stays in the loop."],
  "bugbee report": ["Preparing SARIF export...", "✓ Findings normalized", "✓ Locations and traces attached", "✓ Review history preserved", "", "Ready to write findings.sarif.json"],
  "bugbee ask": ["Preparing a redacted context window...", "✓ Sensitive paths excluded", "✓ Secrets removed before model review", "", "Ask a focused question about this repository."],
} as const;

type TerminalCommand = keyof typeof terminalCommands;

function Terminal({ compact = false }: { compact?: boolean }) {
  const commands = Object.keys(terminalCommands) as TerminalCommand[];
  const [active, setActive] = useState<TerminalCommand>("bugbee hunt");
  const [typed, setTyped] = useState("");
  const lines = terminalCommands[active];

  useEffect(() => {
    setTyped("");
    let cursor = 0;
    const timer = window.setInterval(() => {
      cursor += 1;
      setTyped(active.slice(0, cursor));
      if (cursor >= active.length) window.clearInterval(timer);
    }, 38);
    return () => window.clearInterval(timer);
  }, [active]);

  return (
    <div className={"terminal " + (compact ? "terminalDemo" : "")} aria-label="Interactive Bugbee terminal workspace">
      <div className="terminalChrome"><span><i/><i/><i/></span><span>{compact ? "~/payments-api" : "bugbee / workspace"}</span><span>review</span></div>
      {!compact && <div className="terminalCommands" role="tablist" aria-label="Terminal commands">{commands.map((command) => <button key={command} className={command === active ? "active" : ""} onClick={() => setActive(command)} role="tab" aria-selected={command === active}>{command.replace("bugbee ", "")}</button>)}</div>}
      <div className="terminalCode" aria-live="polite">
        <p><b>$</b> {typed}<span className="cursor"/></p>
        {lines.map((line, index) => line ? <p key={index} className={line.startsWith("✓") ? "terminalGood" : index === lines.length - 1 ? "result" : ""}>{line}</p> : <div className="rule" key={index}/>)}
        <p className="ready"><i/> READY FOR HUMAN REVIEW</p>
      </div>
    </div>
  );
}

export default function Home() {
  const [tilt, setTilt] = useState({ x: 0, y: 0 });
  const [booting, setBooting] = useState(true);

  useEffect(() => {
    const onScroll = () => document.documentElement.style.setProperty("--scroll", String(Math.min(window.scrollY / 900, 1)));
    const boot = window.setTimeout(() => setBooting(false), 900);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => { window.clearTimeout(boot); window.removeEventListener("scroll", onScroll); };
  }, []);

  const move = (event: PointerEvent<HTMLElement>) => {
    const rect = event.currentTarget.getBoundingClientRect();
    setTilt({ x: (event.clientX - rect.left) / rect.width - .5, y: (event.clientY - rect.top) / rect.height - .5 });
  };

  return <main>
    {booting && <div className="bootScreen" role="status" aria-live="polite"><img src="/bugbee-mark-light.png" alt="" /><p>BUGBEE / LOCAL WORKSPACE</p><span className="bootBar"><i/></span></div>}
    <nav className="nav" aria-label="Main navigation">
      <a href="#top" className="brand"><img src="/bugbee-mark-light.png" alt="" /> bugbee</a>
      <div className="navLinks"><a href="#product">Product</a><a href="#workflow">Workflow</a><a href="#architecture">Architecture</a><a href="#access" className="navCta">Private beta</a></div>
    </nav>
    <section id="top" className="hero" onPointerMove={move} onPointerLeave={() => setTilt({ x: 0, y: 0 })}>
      <img className="heroImage" src="/bugbee-security-space.webp" alt="Abstract dark engineered structures with violet signal rings" fetchPriority="high" />
      <div className="fog"/><div className="gridAura"/><div className="rings" aria-hidden="true"><i/><i/><i/></div><div className="particles" aria-hidden="true"/>
      <div className="heroCopy"><p className="eyebrow">PRIVATE BETA · SECURITY INFRASTRUCTURE</p><img className="mark" src="/bugbee-mark-light.png" alt="Bugbee"/><h1>BUGBEE</h1><h2>Terminal-first security workbench</h2><p>A local-first place to investigate application risk, preserve the evidence, and make security decisions with context.</p><div className="actions"><a className="button primary" href="#access">About the beta ↓</a><a className="button" href="#workflow">See the workflow ↓</a></div></div>
      <div className="terminalScene" style={{ transform: "perspective(1100px) rotateX(" + (-tilt.y * 5) + "deg) rotateY(" + (tilt.x * 7) + "deg)" }}><span className="crystal a"/><span className="crystal b"/><span className="crystal c"/><Terminal/></div>
    </section>
    <section className="trust"><p>Security work that stays close to the code and accountable to the team.</p><div><span>LOCAL BY DESIGN</span><span>HUMAN IN COMMAND</span><span>EVIDENCE FIRST</span></div></section>
    <section id="product" className="section product"><div className="sectionHead"><p className="eyebrow">WHY BUGBEE</p><h2>Confidence comes from <em>context.</em></h2><p>Bugbee is designed to help teams investigate carefully—not to pretend difficult security decisions can be fully automated.</p></div><div className="featureGrid">{features.map(([number, title, copy]) => <article key={title}><b>{number}</b><div className="orb"/><h3>{title}</h3><p>{copy}</p></article>)}</div></section>
    <section id="workflow" className="workflow"><p className="eyebrow">THE SECURITY LOOP</p><h2>From code to <em>clarity.</em></h2><div className="pipeline">{workflow.map(([title, copy], index) => <div className="pipelineItem" key={title}><article><small>{String(index + 1).padStart(2, "0")}</small><b>{title}</b><span>{copy}</span></article>{index < workflow.length - 1 && <i/>}</div>)}</div></section>
    <section className="section terminalSection"><div><p className="eyebrow">TERMINAL NATIVE</p><h2>Made for the way engineers <em>think.</em></h2><p>Run a focused investigation without losing the surrounding codebase, context, and review history.</p></div><Terminal compact/></section>
    <section id="architecture" className="architecture"><div className="section"><p className="eyebrow">DESIGNED WITH RESTRAINT</p><div className="sectionHead"><h2>Intelligence with a <em>chain of custody.</em></h2><p>From the repository to the report, each decision remains attributable, evidence-bound, and within the engineer&apos;s control.</p></div><div className="architectureFlow">Developer <i>→</i> Repository <i>→</i> Local index <i>→</i> Security engines <i>→</i> Risk scoring <i>→</i> Optional review <i>→</i> Human approval <i>→</i> SARIF</div></div></section>
    <section id="access" className="access"><div className="accessGlow"/><div><p className="eyebrow">PRIVATE BETA</p><h2>Access is being opened <em>carefully.</em></h2><p>We are working with a small number of teams while we learn where Bugbee is most useful.</p><p className="invite">Private-beta access is currently by invitation.</p></div></section>
  </main>;
}
