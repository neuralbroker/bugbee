import { AgentV2 } from "@bugbee-ai/core/agent"
import { AISDK } from "@bugbee-ai/core/aisdk"
import { Catalog } from "@bugbee-ai/core/catalog"
import { CommandV2 } from "@bugbee-ai/core/command"
import { Credential } from "@bugbee-ai/core/credential"
import { AppNodeBuilder } from "@bugbee-ai/core/effect/app-node-builder"
import { LayerNodePlatform } from "@bugbee-ai/core/effect/app-node-platform"
import { LayerNode } from "@bugbee-ai/core/effect/layer-node"
import { EventV2 } from "@bugbee-ai/core/event"
import { FileSystem } from "@bugbee-ai/core/filesystem"
import { FSUtil } from "@bugbee-ai/core/fs-util"
import { Integration } from "@bugbee-ai/core/integration"
import { Location } from "@bugbee-ai/core/location"
import { Npm } from "@bugbee-ai/core/npm"
import { PluginV2 } from "@bugbee-ai/core/plugin"
import { Reference } from "@bugbee-ai/core/reference"
import { SkillV2 } from "@bugbee-ai/core/skill"
import { Effect, Layer } from "effect"
import { tempLocationLayer } from "../fixture/location"

const npmLayer = Layer.succeed(
  Npm.Service,
  Npm.Service.of({
    add: () => Effect.succeed({ directory: "", entrypoint: undefined }),
    install: () => Effect.void,
    which: () => Effect.succeed(undefined),
  }),
)

export const PluginTestLayer = AppNodeBuilder.build(
  LayerNode.group([
    FileSystem.node,
    FSUtil.node,
    Location.node,
    Npm.node,
    Credential.node,
    EventV2.node,
    LayerNodePlatform.httpClient,
    PluginV2.node,
    AgentV2.node,
    AISDK.node,
    Catalog.node,
    CommandV2.node,
    Integration.node,
    Reference.node,
    SkillV2.node,
  ]),
  [
    [Location.node, tempLocationLayer],
    [Npm.node, npmLayer],
  ],
)
