import { spawn } from "child_process"
import type { HarnessVerifyConfig } from "./types"

export type VerifyResult = {
  command: string
  code: number | null
  output: string
  ms: number
}

function runCommand(command: string, cwd: string, timeoutMs: number): Promise<VerifyResult> {
  const start = Date.now()
  return new Promise((resolve) => {
    const child = spawn(command, {
      cwd,
      shell: true,
      env: process.env,
    })
    let out = ""
    const onData = (chunk: Buffer | string) => {
      out += String(chunk)
      if (out.length > 200_000) out = out.slice(-100_000)
    }
    child.stdout?.on("data", onData)
    child.stderr?.on("data", onData)

    const timer = setTimeout(() => {
      child.kill("SIGTERM")
      setTimeout(() => child.kill("SIGKILL"), 2000)
    }, timeoutMs)

    child.on("close", (code) => {
      clearTimeout(timer)
      resolve({
        command,
        code,
        output: out,
        ms: Date.now() - start,
      })
    })
    child.on("error", (err) => {
      clearTimeout(timer)
      resolve({
        command,
        code: 1,
        output: err.message,
        ms: Date.now() - start,
      })
    })
  })
}

/** M5: run configured verify commands after mutating tools. */
export async function runVerify(
  projectDir: string,
  config: Required<HarnessVerifyConfig>,
): Promise<{ ok: boolean; report: string; results: VerifyResult[] }> {
  if (!config.enabled || config.commands.length === 0) {
    return { ok: true, report: "", results: [] }
  }

  const results: VerifyResult[] = []
  for (const command of config.commands) {
    results.push(await runCommand(command, projectDir, config.timeout_ms))
  }

  const failed = results.filter((r) => r.code !== 0)
  const lines = ["[harness.verify]", `commands=${results.length} failed=${failed.length}`]
  for (const r of results) {
    lines.push(`$ ${r.command}  (exit ${r.code ?? "?"} · ${r.ms}ms)`)
    const body = r.output.trim()
    if (body) {
      const clipped =
        body.length > config.max_output_chars
          ? body.slice(0, config.max_output_chars) + "\n…[truncated]"
          : body
      lines.push(clipped)
    }
  }

  return {
    ok: failed.length === 0,
    report: lines.join("\n"),
    results,
  }
}
