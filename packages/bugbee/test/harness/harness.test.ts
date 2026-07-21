import { describe, expect, test } from "bun:test"
import fs from "fs/promises"
import os from "os"
import path from "path"
import { readHarness } from "../../src/harness/config"
import { loadMemoryFiles, ensureMemoryDir } from "../../src/harness/memory"
import { appendTrace } from "../../src/harness/trace"
import { runVerify } from "../../src/harness/verify"
import { DEFAULT_MAX_STEPS } from "../../src/harness/types"

describe("harness", () => {
  test("readHarness defaults", () => {
    const h = readHarness(undefined)
    expect(h.max_steps).toBe(DEFAULT_MAX_STEPS)
    expect(h.memory.enabled).toBe(true)
    expect(h.verify.enabled).toBe(false)
    expect(h.trace.enabled).toBe(false)
    expect(h.verify.after_tools).toContain("edit")
  })

  test("readHarness merges top-level harness", () => {
    const h = readHarness({
      harness: {
        max_steps: 12,
        verify: { enabled: true, commands: ["echo ok"] },
        trace: { enabled: true },
      },
    } as any)
    expect(h.max_steps).toBe(12)
    expect(h.verify.enabled).toBe(true)
    expect(h.verify.commands).toEqual(["echo ok"])
    expect(h.trace.enabled).toBe(true)
  })

  test("memory load and ensure", async () => {
    const dir = await fs.mkdtemp(path.join(os.tmpdir(), "bugbee-mem-"))
    const mem = await ensureMemoryDir(dir)
    await fs.writeFile(path.join(mem, "note.md"), "prefer bun over npm\n")
    const blocks = await loadMemoryFiles(dir)
    expect(blocks.some((b) => b.includes("prefer bun"))).toBe(true)
    await fs.rm(dir, { recursive: true, force: true })
  })

  test("trace append", async () => {
    const dir = await fs.mkdtemp(path.join(os.tmpdir(), "bugbee-trace-"))
    await appendTrace(dir, {
      t: new Date().toISOString(),
      sessionID: "s1",
      tool: "edit",
      ok: true,
    })
    const text = await fs.readFile(path.join(dir, ".bugbee/harness/trace.jsonl"), "utf8")
    expect(text).toContain('"tool":"edit"')
    await fs.rm(dir, { recursive: true, force: true })
  })

  test("verify runs commands", async () => {
    const dir = await fs.mkdtemp(path.join(os.tmpdir(), "bugbee-ver-"))
    const result = await runVerify(dir, {
      enabled: true,
      commands: ["echo harness-ok"],
      after_tools: ["edit"],
      max_output_chars: 1000,
      timeout_ms: 10_000,
    })
    expect(result.ok).toBe(true)
    expect(result.report).toContain("harness-ok")
    await fs.rm(dir, { recursive: true, force: true })
  })

  test("verify fails on non-zero", async () => {
    const dir = await fs.mkdtemp(path.join(os.tmpdir(), "bugbee-ver2-"))
    const result = await runVerify(dir, {
      enabled: true,
      commands: ["false"],
      after_tools: ["edit"],
      max_output_chars: 1000,
      timeout_ms: 10_000,
    })
    expect(result.ok).toBe(false)
    await fs.rm(dir, { recursive: true, force: true })
  })
})
