"use client";

import { FormEvent, useState } from "react";
import { Check } from "lucide-react";
import { MagneticButton } from "../components/MagneticButton";

export function PrivateBeta() {
  const [submitted, setSubmitted] = useState(false);
  const submit = (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); setSubmitted(true); };
  return <section id="access" className="beta-section"><div className="beta-halo" aria-hidden="true"/><div className="beta-content"><p className="kicker">PRIVATE BETA</p><h2>Join the private beta.</h2><p>Early access for engineering teams shipping secure software.</p>{submitted ? <p className="beta-confirmation"><Check size={18}/> Request captured. We&apos;ll be in touch when access opens.</p> : <form onSubmit={submit}><label className="sr-only" htmlFor="beta-email">Work email</label><input id="beta-email" type="email" required placeholder="you@company.com" autoComplete="email" /><MagneticButton variant="primary">Request access</MagneticButton></form>}<small>No spam. No sales team. Just early access.</small></div></section>;
}
