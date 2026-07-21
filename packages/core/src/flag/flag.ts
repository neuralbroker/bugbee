import { Config } from "effect"

export function truthy(key: string) {
  const value = process.env[key]?.toLowerCase()
  return value === "true" || value === "1"
}

const copy = process.env["BUGBEE_EXPERIMENTAL_DISABLE_COPY_ON_SELECT"]
const fff = process.env["BUGBEE_DISABLE_FFF"]

function enabledByExperimental(key: string) {
  return process.env[key] === undefined ? truthy("BUGBEE_EXPERIMENTAL") : truthy(key)
}

export const Flag = {
  OTEL_EXPORTER_OTLP_ENDPOINT: process.env["OTEL_EXPORTER_OTLP_ENDPOINT"],
  OTEL_EXPORTER_OTLP_HEADERS: process.env["OTEL_EXPORTER_OTLP_HEADERS"],

  BUGBEE_AUTO_HEAP_SNAPSHOT: truthy("BUGBEE_AUTO_HEAP_SNAPSHOT"),
  BUGBEE_GIT_BASH_PATH: process.env["BUGBEE_GIT_BASH_PATH"],
  BUGBEE_CONFIG: process.env["BUGBEE_CONFIG"],
  BUGBEE_CONFIG_CONTENT: process.env["BUGBEE_CONFIG_CONTENT"],
  BUGBEE_DISABLE_AUTOUPDATE: truthy("BUGBEE_DISABLE_AUTOUPDATE"),
  BUGBEE_ALWAYS_NOTIFY_UPDATE: truthy("BUGBEE_ALWAYS_NOTIFY_UPDATE"),
  BUGBEE_DISABLE_PRUNE: truthy("BUGBEE_DISABLE_PRUNE"),
  BUGBEE_DISABLE_TERMINAL_TITLE: truthy("BUGBEE_DISABLE_TERMINAL_TITLE"),
  BUGBEE_SHOW_TTFD: truthy("BUGBEE_SHOW_TTFD"),
  BUGBEE_DISABLE_AUTOCOMPACT: truthy("BUGBEE_DISABLE_AUTOCOMPACT"),
  BUGBEE_DISABLE_MODELS_FETCH: truthy("BUGBEE_DISABLE_MODELS_FETCH"),
  BUGBEE_DISABLE_MOUSE: truthy("BUGBEE_DISABLE_MOUSE"),
  BUGBEE_FAKE_VCS: process.env["BUGBEE_FAKE_VCS"],
  BUGBEE_SERVER_PASSWORD: process.env["BUGBEE_SERVER_PASSWORD"],
  BUGBEE_SERVER_USERNAME: process.env["BUGBEE_SERVER_USERNAME"],
  BUGBEE_DISABLE_FFF: fff === undefined ? process.platform === "win32" : truthy("BUGBEE_DISABLE_FFF"),

  // Experimental
  BUGBEE_EXPERIMENTAL_FILEWATCHER: Config.boolean("BUGBEE_EXPERIMENTAL_FILEWATCHER").pipe(
    Config.withDefault(false),
  ),
  BUGBEE_EXPERIMENTAL_DISABLE_FILEWATCHER: Config.boolean("BUGBEE_EXPERIMENTAL_DISABLE_FILEWATCHER").pipe(
    Config.withDefault(false),
  ),
  BUGBEE_EXPERIMENTAL_DISABLE_COPY_ON_SELECT:
    copy === undefined ? process.platform === "win32" : truthy("BUGBEE_EXPERIMENTAL_DISABLE_COPY_ON_SELECT"),
  BUGBEE_MODELS_URL: process.env["BUGBEE_MODELS_URL"],
  BUGBEE_MODELS_PATH: process.env["BUGBEE_MODELS_PATH"],
  BUGBEE_DB: process.env["BUGBEE_DB"],

  BUGBEE_WORKSPACE_ID: process.env["BUGBEE_WORKSPACE_ID"],
  BUGBEE_EXPERIMENTAL_WORKSPACES: enabledByExperimental("BUGBEE_EXPERIMENTAL_WORKSPACES"),

  // Evaluated at access time (not module load) because tests, the CLI, and
  // external tooling set these env vars at runtime.
  get BUGBEE_DISABLE_PROJECT_CONFIG() {
    return truthy("BUGBEE_DISABLE_PROJECT_CONFIG")
  },
  get BUGBEE_EXPERIMENTAL_REFERENCES() {
    return enabledByExperimental("BUGBEE_EXPERIMENTAL_REFERENCES")
  },
  get BUGBEE_TUI_CONFIG() {
    return process.env["BUGBEE_TUI_CONFIG"]
  },
  get BUGBEE_CONFIG_DIR() {
    return process.env["BUGBEE_CONFIG_DIR"]
  },
  get BUGBEE_PURE() {
    return truthy("BUGBEE_PURE")
  },
  get BUGBEE_PERMISSION() {
    return process.env["BUGBEE_PERMISSION"]
  },
  get BUGBEE_PLUGIN_META_FILE() {
    return process.env["BUGBEE_PLUGIN_META_FILE"]
  },
  get BUGBEE_CLIENT() {
    return process.env["BUGBEE_CLIENT"] ?? "cli"
  },
}
