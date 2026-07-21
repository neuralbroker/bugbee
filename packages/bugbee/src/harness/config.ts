import type { ConfigV1 } from "@bugbee-ai/core/v1/config/config"
import {
  DEFAULT_MAX_STEPS,
  DEFAULT_MEMORY_DIR,
  DEFAULT_VERIFY_TOOLS,
  type HarnessConfig,
  type HarnessMemoryConfig,
  type HarnessTraceConfig,
  type HarnessVerifyConfig,
} from "./types"

type LooseConfig = ConfigV1.Info & { harness?: HarnessConfig }

export function readHarness(config: ConfigV1.Info | undefined): Required<
  Pick<HarnessConfig, "max_steps">
> & {
  memory: Required<HarnessMemoryConfig>
  verify: Required<HarnessVerifyConfig>
  trace: Required<HarnessTraceConfig>
} {
  const root = (config as LooseConfig | undefined)?.harness
  const exp = (config as { experimental?: { harness?: HarnessConfig } } | undefined)?.experimental?.harness
  const h = { ...exp, ...root }

  const verify: Required<HarnessVerifyConfig> = {
    enabled: h?.verify?.enabled ?? false,
    commands: h?.verify?.commands ?? [],
    after_tools: h?.verify?.after_tools ?? [...DEFAULT_VERIFY_TOOLS],
    max_output_chars: h?.verify?.max_output_chars ?? 4000,
    timeout_ms: h?.verify?.timeout_ms ?? 120_000,
  }

  const memory: Required<HarnessMemoryConfig> = {
    enabled: h?.memory?.enabled ?? true,
    dir: h?.memory?.dir ?? DEFAULT_MEMORY_DIR,
  }

  const trace: Required<HarnessTraceConfig> = {
    enabled: h?.trace?.enabled ?? false,
  }

  return {
    max_steps: h?.max_steps ?? DEFAULT_MAX_STEPS,
    memory,
    verify,
    trace,
  }
}
