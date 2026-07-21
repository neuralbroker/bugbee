import fs from "fs/promises"
import path from "path"
import { DEFAULT_TRACE_PATH, type TraceEvent } from "./types"

export async function appendTrace(projectDir: string, event: TraceEvent, relative = DEFAULT_TRACE_PATH): Promise<void> {
  const file = path.isAbsolute(relative) ? relative : path.join(projectDir, relative)
  await fs.mkdir(path.dirname(file), { recursive: true })
  await fs.appendFile(file, JSON.stringify(event) + "\n", "utf8")
}
