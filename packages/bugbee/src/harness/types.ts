export type HarnessVerifyConfig = {
  /** Run verify after mutating tools. Default false until user opts in. */
  enabled?: boolean
  /** Shell commands to run (cwd = project root). */
  commands?: string[]
  /** Tool names that trigger verify. Default: edit, write, apply_patch */
  after_tools?: string[]
  /** Max combined stdout/stderr chars to inject. Default 4000 */
  max_output_chars?: number
  /** Per-command timeout ms. Default 120000 */
  timeout_ms?: number
}

export type HarnessMemoryConfig = {
  /** Load .bugbee/memory/*.md into system context. Default true */
  enabled?: boolean
  /** Relative to project root. Default .bugbee/memory */
  dir?: string
}

export type HarnessTraceConfig = {
  /** Append tool steps to .bugbee/harness/trace.jsonl. Default false */
  enabled?: boolean
}

export type HarnessConfig = {
  /** Default max agent steps for primary agents when agent.steps unset. */
  max_steps?: number
  memory?: HarnessMemoryConfig
  verify?: HarnessVerifyConfig
  trace?: HarnessTraceConfig
}

export type TraceEvent = {
  t: string
  sessionID: string
  tool: string
  callID?: string
  ok: boolean
  ms?: number
  title?: string
  detail?: string
}

export const DEFAULT_VERIFY_TOOLS = ["edit", "write", "apply_patch"] as const
export const DEFAULT_MAX_STEPS = 80
export const DEFAULT_MEMORY_DIR = ".bugbee/memory"
export const DEFAULT_TRACE_PATH = ".bugbee/harness/trace.jsonl"
