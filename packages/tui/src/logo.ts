/** Bugbee wordmark glyphs for TUI + CLI.
 * Special marks used by the renderer:
 *   _ = full block with shadow bg
 *   ^ = half-block mix (▀ with fg+bg)
 *   ~ = top-half shadow only
 *   , = bottom half shadow only (▄)
 */
export const logo = {
  // Muted bee-wing accent (left column)
  left: [
    "      ▄▄▄      ",
    "    ▀▀███▀▀    ",
    "      ▐█▌      ",
    "    ═══════    ",
  ],
  // Bold BUGBEE wordmark
  right: [
    "                              ",
    "█▀▀▄ █  █ █▀▀▀ █▀▀▄ █▀▀▀ █▀▀▀ ",
    "█▀▀▄ █  █ █ ▄▄ █▀▀▄ █▀▀  █▀▀  ",
    "▀▀▀  ▀▀▀▀ ▀▀▀▀ ▀▀▀  ▀▀▀▀ ▀▀▀▀ ",
  ],
}

/** Compact badge used for splash / exit marks */
export const go = {
  left: ["    ", "█▀▀▄", "█▀▀▄", "▀▀▀ "],
  right: ["    ", "█▀▀▄", "█▀▀▄", "▀▀▀ "],
}

export const marks = "_^~,"
