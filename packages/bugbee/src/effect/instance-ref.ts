import { Context } from "effect"
import type { InstanceContext } from "@/project/instance-context"
import type { WorkspaceV2 } from "@bugbee-ai/core/workspace"

export const InstanceRef = Context.Reference<InstanceContext | undefined>("~bugbee/InstanceRef", {
  defaultValue: () => undefined,
})

export const WorkspaceRef = Context.Reference<WorkspaceV2.ID | undefined>("~bugbee/WorkspaceRef", {
  defaultValue: () => undefined,
})
