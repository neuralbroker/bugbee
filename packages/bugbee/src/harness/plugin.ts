import type { Plugin } from "@bugbee-ai/plugin"
import path from "path"
import { readHarness } from "./config"
import { runVerify } from "./verify"
import { appendTrace } from "./trace"

/**
 * Superharness plugin (M3 control hooks + M5 verify + M6 trace).
 * Registered as an internal plugin — no user install required.
 */
export const HarnessPlugin: Plugin = async (input) => {
  const directory = input.directory
  // Config is read per-hook via client so reloads apply
  const getConfig = async () => {
    try {
      const res = await input.client.config.get()
      return readHarness(res.data as any)
    } catch {
      return readHarness(undefined)
    }
  }

  const mutateCount = new Map<string, number>()

  return {
    "tool.execute.after": async (event, output) => {
      const harness = await getConfig()
      const tool = event.tool
      const isMutate = harness.verify.after_tools.includes(tool)

      if (harness.trace.enabled) {
        await appendTrace(directory, {
          t: new Date().toISOString(),
          sessionID: event.sessionID,
          tool,
          callID: event.callID,
          ok: true,
          title: output.title,
        }).catch(() => undefined)
      }

      if (!isMutate || !harness.verify.enabled || harness.verify.commands.length === 0) {
        return
      }

      const n = (mutateCount.get(event.sessionID) ?? 0) + 1
      mutateCount.set(event.sessionID, n)

      // Run verify after every mutating tool when enabled (opt-in via config)
      const result = await runVerify(directory, harness.verify)
      if (!result.report) return

      output.output = [output.output, "", result.report].filter(Boolean).join("\n")
      output.metadata = {
        ...output.metadata,
        harness_verify: {
          ok: result.ok,
          commands: result.results.map((r) => ({ command: r.command, code: r.code, ms: r.ms })),
        },
      }

      if (harness.trace.enabled) {
        await appendTrace(directory, {
          t: new Date().toISOString(),
          sessionID: event.sessionID,
          tool: "harness.verify",
          callID: event.callID,
          ok: result.ok,
          detail: result.results.map((r) => `${r.command}:${r.code}`).join(","),
        }).catch(() => undefined)
      }
    },

    "experimental.chat.system.transform": async (_event, output) => {
      const harness = await getConfig()
      if (!harness.memory.enabled) return

      // Memory files are primarily loaded via Instruction.systemPaths;
      // this injects a short harness banner so the model knows verify may run.
      if (harness.verify.enabled && harness.verify.commands.length) {
        output.system.push(
          [
            "Harness verify is enabled for this project.",
            `After file mutations, these commands may run: ${harness.verify.commands.join(" · ")}`,
            "Fix failures before claiming the task is done.",
          ].join(" "),
        )
      }

      // Point agents at memory dir without dumping content (content via instructions)
      const mem = path.join(directory, harness.memory.dir)
      output.system.push(
        `Optional durable notes live under ${mem} (when present they are loaded as instructions).`,
      )
    },
  }
}
