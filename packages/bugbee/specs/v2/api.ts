// @ts-nocheck

import { Bugbee } from "@bugbee-ai/core"
import { ReadTool } from "@bugbee-ai/core/tools"

const bugbee = Bugbee.make({})

bugbee.tool.add(ReadTool)

bugbee.tool.add({
  name: "bash",
  schema: {
    type: "object",
    properties: {
      command: {
        type: "string",
        description: "The command to run.",
      },
    },
    required: ["command"],
  },
  execute(input, ctx) {},
})

bugbee.auth.add({
  provider: "openai",
  type: "api",
  value: process.env.OPENAI_API_KEY,
})

bugbee.agent.add({
  name: "build",
  permissions: [],
  model: {
    id: "gpt-5-5",
    provider: "openai",
    variant: "xhigh",
  },
})

const sessionID = await bugbee.session.create({
  agent: "build",
})

bugbee.subscribe((event) => {
  console.log(event)
})

await bugbee.session.prompt({
  sessionID,
  text: "hey what is up",
})

await bugbee.session.prompt({
  sessionID,
  text: "what is up with this",
  files: [
    {
      mime: "image/png",
      uri: "data:image/png;base64,xxxx",
    },
  ],
})

await bugbee.session.wait()

console.log(await bugbee.session.messages(sessionID))
