"use client";

import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";
import { useReducedMotion } from "../hooks/useReducedMotion";

export function LoadingScreen() {
  const [visible, setVisible] = useState(true);
  const reduced = useReducedMotion();
  useEffect(() => { if (reduced) { setVisible(false); return; } const timeout = window.setTimeout(() => setVisible(false), 1560); return () => window.clearTimeout(timeout); }, [reduced]);
  return <AnimatePresence>{visible && <motion.div className="loading-screen" initial={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.8 }} role="status" aria-label="Loading Bugbee"><p className="loading-wordmark">Bugbee<span className="terminal-cursor" /></p><div className="boot-lines"><span>&gt; initializing core…</span><span>&gt; loading security engines…</span><span>&gt; mounting filesystem…</span><span>&gt; ready.</span></div><div className="boot-progress"><i /></div></motion.div>}</AnimatePresence>;
}
