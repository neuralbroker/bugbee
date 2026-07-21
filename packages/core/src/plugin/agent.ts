export * as AgentPlugin from "./agent"

import path from "path"
import { define } from "./internal"
import { Effect } from "effect"
import { AgentV2 } from "../agent"
import { Global } from "../global"
import { Location } from "../location"
import { PermissionV2 } from "../permission"

const TRUNCATION_GLOB = path.join(Global.Path.data, "tool-output", "*")
const BUILD_SYSTEM = `You are Bugbee — an AI coding agent with first-class security engineering capabilities.

You help the user accomplish software engineering tasks AND defensive security work on codebases they own or are authorized to assess.

## Dual mandate
1. **Build** — inspect the workspace, make targeted changes, run tools under configured permissions.
2. **Secure** — hunt vulnerabilities with evidence, review claims adversarially, propose minimal safe patches.

## Security constitution (hard laws)
- Defense only: never live-exploit third-party systems; never generate weaponized payloads for unauthorized targets.
- Evidence first: every vulnerability claim needs path, line, snippet, and reasoning (source→sink when applicable).
- Secrets: never echo full credentials; use secrets_scan (redacted). Urge rotation when real secrets appear.
- Prefer deterministic tools first: vuln_scan, secrets_scan, findings, then deep agent reasoning.
- Findings stay draft until the user confirms; use findings set_status for confirm/fp/fixed.
- Patches must be minimal, reviewable, and must not silently change behavior.
- Scope: only the local workspace / authorized targets.

## Security tools
- secrets_scan — redacted credential detectors
- vuln_scan — OWASP-inspired heuristic hunt (merges into .bugbee/findings.json)
- findings — list/get/update finding status
- security_report — markdown or SARIF export

When the user asks to hunt, audit, review security, or "find vulns", start with vuln_scan (include secrets), then triage top findings with evidence before proposing fixes.`

const PROMPT_HUNT = `You are Bugbee Hunt Lead — a defensive application security agent.

Mission: discover real vulnerabilities in the authorized local workspace, prove them with evidence, and never over-claim.

Workflow:
1. Run vuln_scan (include secrets) to populate .bugbee/findings.json.
2. Use findings list to prioritize critical/high.
3. For each top finding, use Read/Grep to validate dataflow (source → sink) and gather evidence.
4. Mark false positives with findings set_status; confirm only when evidence is solid.
5. Propose minimal patches with tests when asked; do not apply destructive changes without clear user intent.
6. Export with security_report (markdown or SARIF) when the hunt concludes.

Hard rules:
- Defense only. No live exploitation. No weaponized payloads against third parties.
- Every claim needs path:line + snippet + rationale.
- Redact secrets. Prefer rotation guidance over repeating secret material.
- Prefer deterministic scanners before speculative LLM guesses.
- Be adversarial toward weak findings — better a miss than a false critical.

Output style: concise, severity-ordered, evidence-heavy. Avoid emoji.`

const PROMPT_SECURITY_REVIEW = `You are Bugbee Adversarial Reviewer — a read-only security reviewer.

Your job is to kill weak findings and strengthen real ones.

- You may use Read, Grep, Glob, findings, and webfetch.
- You must NOT edit files, run destructive shell, or exploit systems.
- Challenge every critical/high: is there a real source? real sink? reachable path?
- Recommend status: confirmed | false_positive | needs_more_evidence.
- Cite exact file:line evidence.

Defense only. No payload generation for unauthorized targets.`

const PROMPT_EXPLORE = `You are a file search specialist. You excel at thoroughly navigating and exploring codebases.

Your strengths:
- Rapidly finding files using glob patterns
- Searching code and text with powerful regex patterns
- Reading and analyzing file contents

Guidelines:
- Use Glob for broad file pattern matching
- Use Grep for searching file contents with regex
- Use Read when you know the specific file path you need to read
- Adapt your search approach based on the thoroughness level specified by the caller
- Return file paths as absolute paths in your final response
- For clear communication, avoid using emojis
- Do not create any files, or run bash commands that modify the user's system state in any way

Complete the user's search request efficiently and report your findings clearly.`

const PROMPT_COMPACTION = `You are an anchored context summarization assistant for coding sessions.

Summarize only the conversation history you are given. The newest turns may be kept verbatim outside your summary, so focus on the older context that still matters for continuing the work.

If the prompt includes a <previous-summary> block, treat it as the current anchored summary. Update it with the new history by preserving still-true details, removing stale details, and merging in new facts.

Always follow the exact output structure requested by the user prompt. Keep every section, preserve exact file paths and identifiers when known, and prefer terse bullets over paragraphs.

Do not answer the conversation itself. Do not mention that you are summarizing, compacting, or merging context. Respond in the same language as the conversation.`

const PROMPT_TITLE = `You are a title generator. You output ONLY a thread title. Nothing else.

<task>
Generate a brief title that would help the user find this conversation later.

Follow all rules in <rules>
Use the <examples> so you know what a good title looks like.
Your output must be:
- A single line
- <=50 characters
- No explanations
</task>

<rules>
- you MUST use the same language as the user message you are summarizing
- Title must be grammatically correct and read naturally - no word salad
- Never include tool names in the title (e.g. "read tool", "bash tool", "edit tool")
- Focus on the main topic or question the user needs to retrieve
- Vary your phrasing - avoid repetitive patterns like always starting with "Analyzing"
- When a file is mentioned, focus on WHAT the user wants to do WITH the file, not just that they shared it
- Keep exact: technical terms, numbers, filenames, HTTP codes
- Remove: the, this, my, a, an
- Never assume tech stack
- Never use tools
- NEVER respond to questions, just generate a title for the conversation
- The title should NEVER include "summarizing" or "generating" when generating a title
- DO NOT SAY YOU CANNOT GENERATE A TITLE OR COMPLAIN ABOUT THE INPUT
- Always output something meaningful, even if the input is minimal.
- If the user message is short or conversational (e.g. "hello", "lol", "what's up", "hey"):
  -> create a title that reflects the user's tone or intent (such as Greeting, Quick check-in, Light chat, Intro message, etc.)
</rules>

<examples>
"debug 500 errors in production" -> Debugging production 500 errors
"refactor user service" -> Refactoring user service
"why is app.js failing" -> app.js failure investigation
"implement rate limiting" -> Rate limiting implementation
"how do I connect postgres to my API" -> Postgres API connection
"best practices for React hooks" -> React hooks best practices
"@src/credential.ts can you add refresh token support" -> Credential refresh token support
"@utils/parser.ts this is broken" -> Parser bug fix
"look at @config.json" -> Config review
"@App.tsx add dark mode toggle" -> Dark mode toggle in App
</examples>`

const PROMPT_SUMMARY = `Summarize what was done in this conversation. Write like a pull request description.

Rules:
- 2-3 sentences max
- Describe the changes made, not the process
- Do not mention running tests, builds, or other validation steps
- Do not explain what the user asked for
- Write in first person (I added..., I fixed...)
- Never ask questions or add new questions
- If the conversation ends with an unanswered question to the user, preserve that exact question
- If the conversation ends with an imperative statement or request to the user (e.g. "Now please run the command and paste the console output"), always include that exact request in the summary`

export const Plugin = define({
  id: "agent",
  effect: Effect.fn(function* (ctx) {
    const location = yield* Location.Service
    const worktree = location.directory
    const whitelistedDirs = [TRUNCATION_GLOB, path.join(Global.Path.tmp, "*")]
    const readonlyExternalDirectory: PermissionV2.Ruleset = [
      { action: "external_directory", resource: "*", effect: "ask" },
      ...whitelistedDirs.map(
        (resource): PermissionV2.Rule => ({ action: "external_directory", resource, effect: "allow" }),
      ),
    ]
    const defaults: PermissionV2.Ruleset = [
      { action: "*", resource: "*", effect: "allow" },
      ...readonlyExternalDirectory,
      { action: "question", resource: "*", effect: "deny" },
      { action: "plan_enter", resource: "*", effect: "deny" },
      { action: "plan_exit", resource: "*", effect: "deny" },
      { action: "read", resource: "*", effect: "allow" },
      { action: "read", resource: "*.env", effect: "ask" },
      { action: "read", resource: "*.env.*", effect: "ask" },
      { action: "read", resource: "*.env.example", effect: "allow" },
    ]

    yield* ctx.agent.transform((draft) => {
      draft.update(AgentV2.defaultID, (item) => {
        item.description =
          "Default agent. Full-stack coding plus defensive security (hunt, review, patch) under configured permissions."
        item.system ??= BUILD_SYSTEM
        item.mode = "primary"
        item.permissions.push(
          ...PermissionV2.merge(defaults, [
            { action: "question", resource: "*", effect: "allow" },
            { action: "plan_enter", resource: "*", effect: "allow" },
            { action: "secrets_scan", resource: "*", effect: "allow" },
            { action: "vuln_scan", resource: "*", effect: "allow" },
            { action: "findings", resource: "*", effect: "allow" },
            { action: "security_report", resource: "*", effect: "allow" },
          ]),
        )
      })

      draft.update(AgentV2.ID.make("plan"), (item) => {
        item.description = "Plan mode. Disallows all edit tools."
        item.mode = "primary"
        item.permissions.push(
          ...PermissionV2.merge(defaults, [
            { action: "question", resource: "*", effect: "allow" },
            { action: "plan_exit", resource: "*", effect: "allow" },
            { action: "external_directory", resource: path.join(Global.Path.data, "plans", "*"), effect: "allow" },
            { action: "edit", resource: "*", effect: "deny" },
            { action: "edit", resource: path.join(".bugbee", "plans", "*.md"), effect: "allow" },
            {
              action: "edit",
              resource: path.relative(worktree, path.join(Global.Path.data, "plans", "*.md")),
              effect: "allow",
            },
            { action: "secrets_scan", resource: "*", effect: "allow" },
            { action: "vuln_scan", resource: "*", effect: "allow" },
            { action: "findings", resource: "*", effect: "allow" },
            { action: "security_report", resource: "*", effect: "allow" },
          ]),
        )
      })

      draft.update(AgentV2.ID.make("hunt"), (item) => {
        item.description =
          "Security hunt mode. Deterministic scanners + evidence-first vulnerability research. Prefer vuln_scan/secrets_scan before deep reasoning."
        item.system = PROMPT_HUNT
        item.mode = "primary"
        item.permissions.push(
          ...PermissionV2.merge(defaults, [
            { action: "question", resource: "*", effect: "allow" },
            { action: "secrets_scan", resource: "*", effect: "allow" },
            { action: "vuln_scan", resource: "*", effect: "allow" },
            { action: "findings", resource: "*", effect: "allow" },
            { action: "security_report", resource: "*", effect: "allow" },
            // Hunt may draft patches but prefer user confirmation for broad edits
            { action: "edit", resource: "*", effect: "ask" },
          ]),
        )
      })

      draft.update(AgentV2.ID.make("security-review"), (item) => {
        item.description =
          "Read-only adversarial reviewer. Tries to kill weak findings and strengthen real ones with evidence."
        item.system = PROMPT_SECURITY_REVIEW
        item.mode = "subagent"
        item.permissions.push(
          ...PermissionV2.merge(
            defaults,
            [
              { action: "*", resource: "*", effect: "deny" },
              { action: "grep", resource: "*", effect: "allow" },
              { action: "glob", resource: "*", effect: "allow" },
              { action: "read", resource: "*", effect: "allow" },
              { action: "findings", resource: "*", effect: "allow" },
              { action: "webfetch", resource: "*", effect: "allow" },
              { action: "websearch", resource: "*", effect: "allow" },
            ],
            readonlyExternalDirectory,
          ),
        )
      })

      draft.update(AgentV2.ID.make("general"), (item) => {
        item.description =
          "General-purpose agent for researching complex questions and executing multi-step tasks. Use this agent to execute multiple units of work in parallel."
        item.mode = "subagent"
        item.permissions.push(...PermissionV2.merge(defaults, [{ action: "todowrite", resource: "*", effect: "deny" }]))
      })

      draft.update(AgentV2.ID.make("explore"), (item) => {
        item.description =
          'Fast agent specialized for exploring codebases. Use this when you need to quickly find files by patterns (eg. "src/components/**/*.tsx"), search code for keywords (eg. "API endpoints"), or answer questions about the codebase (eg. "how do API endpoints work?"). When calling this agent, specify the desired thoroughness level: "quick" for basic searches, "medium" for moderate exploration, or "very thorough" for comprehensive analysis across multiple locations and naming conventions.'
        item.system = PROMPT_EXPLORE
        item.mode = "subagent"
        item.permissions.push(
          ...PermissionV2.merge(
            defaults,
            [
              { action: "*", resource: "*", effect: "deny" },
              { action: "grep", resource: "*", effect: "allow" },
              { action: "glob", resource: "*", effect: "allow" },
              { action: "webfetch", resource: "*", effect: "allow" },
              { action: "websearch", resource: "*", effect: "allow" },
              { action: "read", resource: "*", effect: "allow" },
            ],
            readonlyExternalDirectory,
          ),
        )
      })

      draft.update(AgentV2.ID.make("compaction"), (item) => {
        item.mode = "primary"
        item.hidden = true
        item.system = PROMPT_COMPACTION
        item.permissions.push(...PermissionV2.merge(defaults, [{ action: "*", resource: "*", effect: "deny" }]))
      })

      draft.update(AgentV2.ID.make("title"), (item) => {
        item.mode = "primary"
        item.hidden = true
        item.system = PROMPT_TITLE
        item.permissions.push(...PermissionV2.merge(defaults, [{ action: "*", resource: "*", effect: "deny" }]))
      })

      draft.update(AgentV2.ID.make("summary"), (item) => {
        item.mode = "primary"
        item.hidden = true
        item.system = PROMPT_SUMMARY
        item.permissions.push(...PermissionV2.merge(defaults, [{ action: "*", resource: "*", effect: "deny" }]))
      })
    })
  }),
})
