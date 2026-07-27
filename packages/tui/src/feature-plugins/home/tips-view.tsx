import type { TuiPluginApi } from "@bugbee-ai/plugin/tui"
import { createMemo, For, Show, type Accessor } from "solid-js"
import { DEFAULT_THEMES, useTheme } from "../../context/theme"
import { useCommandShortcut } from "../../keymap"

const themeCount = Object.keys(DEFAULT_THEMES).length

type TipPart = { text: string; highlight: boolean }
type TipShortcut = Accessor<string>
type Shortcuts = {
  agentCycle: TipShortcut
  childFirst: TipShortcut
  childNext: TipShortcut
  childPrevious: TipShortcut
  commandList: TipShortcut
  editorOpen: TipShortcut
  helpShow: TipShortcut
  inputClear: TipShortcut
  inputNewline: TipShortcut
  inputPaste: TipShortcut
  inputUndo: TipShortcut
  leader: TipShortcut
  messagesCopy: TipShortcut
  messagesFirst: TipShortcut
  messagesLast: TipShortcut
  messagesPageDown: TipShortcut
  messagesPageUp: TipShortcut
  messagesToggleConceal: TipShortcut
  modelCycleRecent: TipShortcut
  modelList: TipShortcut
  sessionExport: TipShortcut
  sessionInterrupt: TipShortcut
  sessionList: TipShortcut
  sessionNew: TipShortcut
  sessionParent: TipShortcut
  sessionPinToggle: TipShortcut
  sessionQuickSwitch1: TipShortcut
  sessionQuickSwitch9: TipShortcut
  sessionSidebarToggle: TipShortcut
  sessionTimeline: TipShortcut
  statusView: TipShortcut
  terminalSuspend: TipShortcut
  themeList: TipShortcut
}
type Tip = string | ((shortcuts: Shortcuts) => string | undefined)

function parse(tip: string): TipPart[] {
  const parts: TipPart[] = []
  const regex = /\{highlight\}(.*?)\{\/highlight\}/g
  const found = Array.from(tip.matchAll(regex))
  const state = found.reduce(
    (acc, match) => {
      const start = match.index ?? 0
      if (start > acc.index) {
        acc.parts.push({ text: tip.slice(acc.index, start), highlight: false })
      }
      acc.parts.push({ text: match[1], highlight: true })
      acc.index = start + match[0].length
      return acc
    },
    { parts, index: 0 },
  )

  if (state.index < tip.length) {
    parts.push({ text: tip.slice(state.index), highlight: false })
  }

  return parts
}

const NO_MODELS_TIP = "Add a provider with {highlight}/connect{/highlight}"
const NO_MODELS_PARTS = parse(NO_MODELS_TIP)

function shortcutText(value: string) {
  return `{highlight}${value}{/highlight}`
}

function commandText(command: string, shortcut: string) {
  if (!shortcut) return shortcutText(command)
  return `${shortcutText(command)} · ${shortcutText(shortcut)}`
}

function press(shortcut: string, text: string) {
  if (!shortcut) return undefined
  return `${shortcutText(shortcut)} ${text}`
}

function configShortcut(api: TuiPluginApi, command: string): TipShortcut {
  return () =>
    api.tuiConfig.keybinds
      .get(command)
      .map((binding) => api.keys.formatSequence(Array.from(api.keymap.parseKeySequence(binding.key))))
      .filter(Boolean)
      .join(", ")
}

function KeybindStrip(props: { agent: string; commands: string }) {
  const theme = useTheme().theme
  const items = [
    props.agent ? { key: props.agent, label: "agents" } : undefined,
    props.commands ? { key: props.commands, label: "commands" } : undefined,
  ].filter(Boolean) as { key: string; label: string }[]

  return (
    <Show when={items.length > 0}>
      <box flexDirection="row" gap={2} flexShrink={0} paddingBottom={1}>
        <For each={items}>
          {(item) => (
            <box flexDirection="row" gap={1} flexShrink={0}>
              <text flexShrink={0} style={{ fg: theme.text }}>
                {item.key}
              </text>
              <text flexShrink={0} style={{ fg: theme.textMuted }}>
                {item.label}
              </text>
            </box>
          )}
        </For>
      </box>
    </Show>
  )
}

function TipLine(props: { parts: TipPart[] }) {
  const theme = useTheme().theme
  return (
    <box flexDirection="row" maxWidth="100%" gap={1}>
      <text flexShrink={0} style={{ fg: theme.textMuted }}>
        tip
      </text>
      <text flexShrink={1} wrapMode="word">
        <For each={props.parts}>
          {(part) => <span style={{ fg: part.highlight ? theme.text : theme.textMuted }}>{part.text}</span>}
        </For>
      </text>
    </box>
  )
}

export function Tips(props: { api: TuiPluginApi; connected?: boolean }) {
  const tipOffset = Math.random()
  const shortcuts: Shortcuts = {
    agentCycle: useCommandShortcut("agent.cycle"),
    childFirst: configShortcut(props.api, "session.child.first"),
    childNext: configShortcut(props.api, "session.child.next"),
    childPrevious: configShortcut(props.api, "session.child.previous"),
    commandList: useCommandShortcut("command.palette.show"),
    editorOpen: useCommandShortcut("prompt.editor"),
    helpShow: useCommandShortcut("help.show"),
    inputClear: useCommandShortcut("prompt.clear"),
    inputNewline: useCommandShortcut("input.newline"),
    inputPaste: useCommandShortcut("prompt.paste"),
    inputUndo: useCommandShortcut("input.undo"),
    leader: configShortcut(props.api, "leader"),
    messagesCopy: configShortcut(props.api, "messages.copy"),
    messagesFirst: configShortcut(props.api, "session.first"),
    messagesLast: configShortcut(props.api, "session.last"),
    messagesPageDown: configShortcut(props.api, "session.page.down"),
    messagesPageUp: configShortcut(props.api, "session.page.up"),
    messagesToggleConceal: configShortcut(props.api, "session.toggle.conceal"),
    modelCycleRecent: useCommandShortcut("model.cycle_recent"),
    modelList: useCommandShortcut("model.list"),
    sessionExport: configShortcut(props.api, "session.export"),
    sessionInterrupt: configShortcut(props.api, "session.interrupt"),
    sessionList: useCommandShortcut("session.list"),
    sessionNew: useCommandShortcut("session.new"),
    sessionParent: configShortcut(props.api, "session.parent"),
    sessionPinToggle: configShortcut(props.api, "session.pin.toggle"),
    sessionQuickSwitch1: useCommandShortcut("session.quick_switch.1"),
    sessionQuickSwitch9: useCommandShortcut("session.quick_switch.9"),
    sessionSidebarToggle: configShortcut(props.api, "session.sidebar.toggle"),
    sessionTimeline: configShortcut(props.api, "session.timeline"),
    statusView: useCommandShortcut("bugbee.status"),
    terminalSuspend: useCommandShortcut("terminal.suspend"),
    themeList: useCommandShortcut("theme.switch"),
  }

  const tip = createMemo(() => {
    if (props.connected === false) return NO_MODELS_TIP
    const tips = [...TIPS, process.platform !== "win32" ? TERMINAL_SUSPEND_TIP : INPUT_UNDO_TIP].flatMap((item) => {
      const value = typeof item === "string" ? item : item(shortcuts)
      return value ? [value] : []
    })
    return tips[Math.floor(tipOffset * tips.length)] ?? NO_MODELS_TIP
  }, NO_MODELS_TIP)

  const parts = createMemo(() => {
    const value = tip()
    if (typeof value === "string") return parse(value)
    return NO_MODELS_PARTS
  }, NO_MODELS_PARTS)

  // Stable keybind chrome above the rotating tip (always-available actions first).
  // Fallbacks match default keybinds when bindings are empty/unformatted.
  const agentKey = createMemo(() => shortcuts.agentCycle() || "tab")
  const commandKey = createMemo(() => shortcuts.commandList() || "ctrl+p")

  return (
    <box flexDirection="column" maxWidth="100%" width="100%" alignItems="flex-start">
      <KeybindStrip agent={agentKey()} commands={commandKey()} />
      <TipLine parts={parts()} />
    </box>
  )
}

const TIPS: Tip[] = [
  "Type {highlight}@{/highlight} to attach files",
  "Prefix {highlight}!{/highlight} for shell ({highlight}!git status{/highlight})",
  (shortcuts) => press(shortcuts.agentCycle(), "cycle build / plan"),
  "Use {highlight}/undo{/highlight} to revert last turn",
  "Use {highlight}/redo{/highlight} to restore undone work",
  "Drag images or PDFs into the terminal",
  (shortcuts) => press(shortcuts.inputPaste(), "paste from clipboard"),
  (shortcuts) => `Open editor · ${commandText("/editor", shortcuts.editorOpen())}`,
  "Run {highlight}/init{/highlight} for project rules",
  (shortcuts) => `Models · ${commandText("/models", shortcuts.modelList())}`,
  (shortcuts) => `Themes · ${commandText("/themes", shortcuts.themeList())} (${themeCount})`,
  (shortcuts) => `New session · ${commandText("/new", shortcuts.sessionNew())}`,
  (shortcuts) => `Sessions · ${commandText("/sessions", shortcuts.sessionList())}`,
  (shortcuts) => press(shortcuts.sessionPinToggle(), "pin session in list"),
  "Run {highlight}/compact{/highlight} near context limits",
  (shortcuts) => `Export · ${commandText("/export", shortcuts.sessionExport())}`,
  (shortcuts) => press(shortcuts.messagesCopy(), "copy last reply"),
  (shortcuts) => press(shortcuts.commandList(), "command palette"),
  "Run {highlight}/connect{/highlight} for providers",
  (shortcuts) => press(shortcuts.modelCycleRecent(), "recent models"),
  (shortcuts) => press(shortcuts.sessionSidebarToggle(), "toggle sidebar"),
  (shortcuts) => press(shortcuts.inputNewline(), "newline in prompt"),
  (shortcuts) => press(shortcuts.sessionInterrupt(), "stop generation"),
  "Switch to {highlight}plan{/highlight} for read-only planning",
  "Use {highlight}@review{/highlight} for a read-only review pass",
  "Config: {highlight}bugbee.json{/highlight} + {highlight}tui.json{/highlight}",
  "Global TUI: {highlight}~/.config/bugbee/tui.json{/highlight}",
  "Commands live in {highlight}.bugbee/commands/{/highlight}",
  "Agents live in {highlight}.bugbee/agents/{/highlight}",
  "Memory notes: {highlight}.bugbee/memory/{/highlight}",
  "Run {highlight}bugbee doctor{/highlight} for install health",
  "Harness verify: set {highlight}harness.verify{/highlight} in config",
  'Theme: {highlight}"bugbee"{/highlight} or {highlight}"transparent"{/highlight}',
  (shortcuts) => `Help · ${commandText("/help", shortcuts.helpShow())}`,
  "Use {highlight}/review{/highlight} for branch or PR review",
  "Use {highlight}/doctor{/highlight} for a readiness checklist",
  "Use {highlight}/verify{/highlight} to run tests after changes",
]

const INPUT_UNDO_TIP: Tip = (shortcuts) => press(shortcuts.inputUndo(), "undo prompt edits")
const TERMINAL_SUSPEND_TIP: Tip = (shortcuts) => press(shortcuts.terminalSuspend(), "suspend terminal")
