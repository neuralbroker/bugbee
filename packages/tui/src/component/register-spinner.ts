import { getComponentCatalogue } from "@opentui/solid/components"
import { registerSpinner } from "opentui-spinner/solid"

export function registerBugbeeSpinner() {
  if (!getComponentCatalogue().spinner) registerSpinner()
}
