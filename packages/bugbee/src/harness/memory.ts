import fs from "fs/promises"
import path from "path"
import { DEFAULT_MEMORY_DIR } from "./types"

/** Load markdown memory notes for superharness context (M2). */
export async function loadMemoryFiles(projectDir: string, dir = DEFAULT_MEMORY_DIR): Promise<string[]> {
  const root = path.isAbsolute(dir) ? dir : path.join(projectDir, dir)
  let entries: string[]
  try {
    entries = await fs.readdir(root)
  } catch {
    return []
  }

  const files = entries
    .filter((name) => name.endsWith(".md") || name.endsWith(".txt"))
    .sort((a, b) => a.localeCompare(b))
    .slice(0, 20)

  const blocks: string[] = []
  for (const name of files) {
    const full = path.join(root, name)
    try {
      const text = await fs.readFile(full, "utf8")
      if (!text.trim()) continue
      // Cap each file to keep context bounded
      const body = text.length > 8000 ? text.slice(0, 8000) + "\n…[truncated]" : text
      blocks.push(`Memory from: ${path.relative(projectDir, full)}\n${body}`)
    } catch {
      // skip unreadable
    }
  }
  return blocks
}

export async function ensureMemoryDir(projectDir: string, dir = DEFAULT_MEMORY_DIR): Promise<string> {
  const root = path.isAbsolute(dir) ? dir : path.join(projectDir, dir)
  await fs.mkdir(root, { recursive: true })
  const readme = path.join(root, "README.md")
  try {
    await fs.access(readme)
  } catch {
    await fs.writeFile(
      readme,
      [
        "# Bugbee memory",
        "",
        "Markdown files in this directory are injected into agent system context",
        "when harness memory is enabled (default).",
        "",
        "Keep notes short and factual: architecture decisions, constraints, known traps.",
        "",
      ].join("\n"),
      "utf8",
    )
  }
  return root
}
