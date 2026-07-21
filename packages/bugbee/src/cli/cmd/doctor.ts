import path from "path"
import fs from "fs/promises"
import os from "os"
import { cmd } from "./cmd"
import { UI } from "../ui"
import { Global } from "@bugbee-ai/core/global"
import { InstallationVersion } from "@bugbee-ai/core/installation/version"

type Check = {
  name: string
  ok: boolean
  detail: string
}

/**
 * Bugbee-only readiness check. OpenCode does not ship this command.
 * Offline, no LLM required — validates install identity, paths, and config surface.
 */
export const DoctorCommand = cmd({
  command: "doctor",
  describe: "check Bugbee install health (paths, config, branding)",
  builder: (yargs) =>
    yargs.option("json", {
      type: "boolean",
      default: false,
      describe: "machine-readable output",
    }),
  handler: async (args) => {
    const checks: Check[] = []

    checks.push({
      name: "identity",
      ok: true,
      detail: `brand=bugbee version=${InstallationVersion}`,
    })

    const paths = [
      ["config", Global.Path.config],
      ["data", Global.Path.data],
      ["cache", Global.Path.cache],
      ["state", Global.Path.state],
      ["log", Global.Path.log],
    ] as const

    for (const [label, dir] of paths) {
      try {
        await fs.mkdir(dir, { recursive: true })
        const st = await fs.stat(dir)
        checks.push({
          name: `path.${label}`,
          ok: st.isDirectory(),
          detail: dir,
        })
      } catch (error) {
        checks.push({
          name: `path.${label}`,
          ok: false,
          detail: `${dir} (${error instanceof Error ? error.message : String(error)})`,
        })
      }
    }

    // Config discovery in cwd + home
    const cwd = process.cwd()
    const candidates = [
      path.join(cwd, "bugbee.json"),
      path.join(cwd, "bugbee.jsonc"),
      path.join(cwd, ".bugbee", "bugbee.json"),
      path.join(cwd, ".bugbee", "bugbee.jsonc"),
      path.join(Global.Path.config, "bugbee.json"),
      path.join(Global.Path.config, "bugbee.jsonc"),
      path.join(Global.Path.config, "config.json"),
    ]
    const found: string[] = []
    for (const file of candidates) {
      try {
        await fs.access(file)
        found.push(file)
      } catch {
        // missing is fine
      }
    }
    checks.push({
      name: "config.files",
      ok: true,
      detail: found.length ? found.join(", ") : "none yet (run /connect or edit bugbee.json)",
    })

    // Brand guard: XDG app dir must be bugbee, not opencode
    const appOk = Global.Path.config.includes(`${path.sep}bugbee`) || Global.Path.config.endsWith("bugbee")
    checks.push({
      name: "brand.paths",
      ok: appOk,
      detail: appOk ? "XDG paths use bugbee" : `unexpected config root: ${Global.Path.config}`,
    })

    checks.push({
      name: "runtime",
      ok: true,
      detail: `node=${process.version} platform=${process.platform}/${process.arch} home=${os.homedir()}`,
    })

    const failed = checks.filter((c) => !c.ok)
    if (args.json) {
      process.stdout.write(
        JSON.stringify(
          {
            brand: "bugbee",
            version: InstallationVersion,
            ok: failed.length === 0,
            checks,
          },
          null,
          2,
        ) + "\n",
      )
      if (failed.length) process.exitCode = 1
      return
    }

    UI.println(UI.Style.TEXT_NORMAL_BOLD + "bugbee doctor" + UI.Style.TEXT_NORMAL)
    UI.println(`version ${InstallationVersion}`)
    UI.empty()
    for (const check of checks) {
      const mark = check.ok ? UI.Style.TEXT_SUCCESS_BOLD + "ok  " : UI.Style.TEXT_DANGER_BOLD + "fail"
      UI.println(`${mark}${UI.Style.TEXT_NORMAL} ${check.name.padEnd(16)} ${check.detail}`)
    }
    UI.empty()
    if (failed.length) {
      UI.println(UI.Style.TEXT_DANGER_BOLD + `${failed.length} check(s) failed` + UI.Style.TEXT_NORMAL)
      process.exitCode = 1
      return
    }
    UI.println(UI.Style.TEXT_SUCCESS_BOLD + "All checks passed." + UI.Style.TEXT_NORMAL)
    UI.println("Next: ./bin/bugbee   or   bun run dev")
  },
})
