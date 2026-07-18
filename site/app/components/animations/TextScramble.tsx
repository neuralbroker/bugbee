"use client";

import { useCallback, useRef, useState } from "react";

const CHARS = "!<>-_\\/[]{}—=+*^?#________";

interface TextScrambleProps {
  text: string;
  className?: string;
}

export function TextScramble({ text, className }: TextScrambleProps) {
  const [display, setDisplay] = useState(text);
  const frameRef = useRef(0);
  const queueRef = useRef<
    { from: string; to: string; start: number; end: number; char?: string }[]
  >([]);

  const scramble = useCallback(() => {
    const old = display;
    const length = Math.max(old.length, text.length);
    const queue: typeof queueRef.current = [];

    for (let i = 0; i < length; i++) {
      const from = old[i] || "";
      const to = text[i] || "";
      const start = Math.floor(Math.random() * 20);
      const end = start + Math.floor(Math.random() * 20);
      queue.push({ from, to, start, end });
    }

    queueRef.current = queue;
    cancelAnimationFrame(frameRef.current);

    let frame = 0;
    const update = () => {
      let output = "";
      let complete = 0;

      for (let i = 0; i < queueRef.current.length; i++) {
        const { from, to, start, end, char } = queueRef.current[i];
        if (frame >= end) {
          complete++;
          output += to;
        } else if (frame >= start) {
          if (!char || Math.random() < 0.28) {
            queueRef.current[i].char =
              CHARS[Math.floor(Math.random() * CHARS.length)];
          }
          output += queueRef.current[i].char;
        } else {
          output += from;
        }
      }

      setDisplay(output);

      if (complete < queueRef.current.length) {
        frame++;
        frameRef.current = requestAnimationFrame(update);
      }
    };

    frameRef.current = requestAnimationFrame(update);
  }, [display, text]);

  return (
    <span
      className={className}
      onMouseEnter={scramble}
      onFocus={scramble}
    >
      {display}
    </span>
  );
}
