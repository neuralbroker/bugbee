import { cmd } from "./cmd"
import { runDoctor } from "./doctor"
import { UI } from "../ui"

export const SetupCommand = cmd({
  command: "setup",
  describe: "check readiness and show the first project steps",
  builder: (yargs) =>
    yargs.option("json", {
      type: "boolean",
      default: false,
      describe: "machine-readable readiness output",
    }),
  handler: async (args) => {
    if (args.json) {
      await runDoctor({ json: true })
      return
    }

    UI.println(UI.Style.TEXT_NORMAL_BOLD + "bugbee setup" + UI.Style.TEXT_NORMAL)
    UI.println("Checking your local Bugbee installation and project readiness...")
    UI.empty()
    await runDoctor({ json: false })
    UI.empty()
    UI.println(UI.Style.TEXT_NORMAL_BOLD + "First project steps" + UI.Style.TEXT_NORMAL)
    UI.println("1. Run bugbee from the project directory.")
    UI.println("2. Use /connect to configure an AI provider.")
    UI.println("3. Use /doctor to check project readiness.")
    UI.println("4. Use /init to create project-specific AGENTS.md guidance.")
  },
})
