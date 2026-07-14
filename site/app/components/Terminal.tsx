"use client";

import { FormEvent, useEffect, useRef, useState } from "react";
import { Command, CornerDownLeft } from "lucide-react";
import { cn } from "../lib/utils";
import { useReducedMotion } from "../hooks/useReducedMotion";

const responses = {
  "bugbee hunt": ["Initializing local analysis…", "████████████████", "Repository indexed · 218 files", "OWASP rules loaded", "Secrets engine ready", "AI review queue prepared", "", "42 findings · 12 high · 18 medium · 12 low", "Ready for human review."],
  "bugbee findings": ["Findings in the current workspace", "HIGH    src/auth/session.ts:74    unchecked redirect", "HIGH    api/token/route.ts:31     secret in response path", "MEDIUM  lib/parse.ts:118          unbounded parser input", "", "Evidence is attached to every finding."],
  "bugbee review": ["Opening review queue…", "Evidence graph loaded", "Risk score: 8.2 / 10", "Model rationale: available", "", "Human decision required before export."],
  "bugbee report": ["Preparing SARIF 2.1.0 export…", "Locations normalized", "Evidence references preserved", "Review state included", "", "report.sarif is ready."],
  "bugbee ask": ["Context window prepared locally.", "Secrets and ignored paths are excluded.", "", "Ask: why is session.ts marked high risk?"],
} as const;

type KnownCommand = keyof typeof responses;
type HistoryItem = { command: string; lines: readonly string[] };

interface TerminalProps {
  commands?: string[];
  interactive?: boolean;
  height?: string;
  compact?: boolean;
}

function isKnownCommand(value: string): value is KnownCommand {
  return value in responses;
}

export function Terminal({ commands = ["bugbee hunt"], interactive = true, height = "500px", compact = false }: TerminalProps) {
  const initial = commands[0] && isKnownCommand(commands[0]) ? commands[0] : "bugbee hunt";
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [value, setValue] = useState<string>(initial);
  const [typing, setTyping] = useState("");
  const [busy, setBusy] = useState(true);
  const outputRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();

  const run = (raw: string) => {
    const command = raw.trim().toLowerCase();
    const lines = isKnownCommand(command) ? responses[command] : ["Command not found.", "Try: bugbee hunt · findings · review · report · ask"];
    setHistory((items) => [...items, { command, lines }]);
    setValue("");
    setBusy(false);
  };

  useEffect(() => {
    const command = initial;
    if (reduced) { setTyping(command); setBusy(false); run(command); return; }
    setTyping(""); setBusy(true);
    let index = 0;
    const interval = window.setInterval(() => {
      index += 1;
      setTyping(command.slice(0, index));
      if (index >= command.length) { window.clearInterval(interval); window.setTimeout(() => run(command), 260); }
    }, 30);
    return () => window.clearInterval(interval);
  // Initial terminal boot only; commands are intentionally user-driven afterwards.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [initial, reduced]);

  useEffect(() => { outputRef.current?.scrollTo({ top: outputRef.current.scrollHeight, behavior: reduced ? "auto" : "smooth" }); }, [history, reduced]);
  const submit = (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); if (value.trim()) run(value); };
  const available = Object.keys(responses) as KnownCommand[];

  return <div className={cn("terminal", compact && "terminal-compact")} style={{ "--terminal-height": height } as React.CSSProperties} aria-label="Interactive Bugbee terminal">
    <div className="terminal-header"><div className="terminal-lights"><i/><i/><i/></div><span><Command size={13} /> {compact ? "~/payments-api" : "bugbee / local workspace"}</span><span className="terminal-state">local</span></div>
    {!compact && <div className="terminal-commandbar" role="tablist" aria-label="Demo terminal commands">{available.map((command) => <button key={command} type="button" onClick={() => run(command)}>{command.replace("bugbee ", "")}</button>)}</div>}
    <div ref={outputRef} className="terminal-output" aria-live="polite">
      {busy && <p><b>$</b> {typing}<span className="terminal-cursor" /></p>}
      {history.map((item, index) => <div className="terminal-entry" key={`${item.command}-${index}`}><p><b>$</b> {item.command}</p>{item.lines.map((line, lineIndex) => line ? <p key={lineIndex} className={line.includes("HIGH") ? "terminal-danger" : line.includes("ready") || line.includes("Ready") ? "terminal-success" : ""}>{line}</p> : <div key={lineIndex} className="terminal-rule" />)}</div>)}
      {!busy && <p className="terminal-prompt"><b>$</b> <span className="terminal-cursor" /></p>}
    </div>
    {interactive && <form className="terminal-input" onSubmit={submit}><span>$</span><input aria-label="Enter a Bugbee demo command" value={value} onChange={(event) => setValue(event.target.value)} placeholder="bugbee hunt" autoComplete="off" /><button type="submit" aria-label="Run command"><CornerDownLeft size={16} /></button></form>}
  </div>;
}
