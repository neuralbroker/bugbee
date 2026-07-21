export * from "./client.js"
export * from "./server.js"

import { createBugbeeClient } from "./client.js"
import { createBugbeeServer } from "./server.js"
import type { ServerOptions } from "./server.js"

export async function createBugbee(options?: ServerOptions) {
  const server = await createBugbeeServer({
    ...options,
  })

  const client = createBugbeeClient({
    baseUrl: server.url,
  })

  return {
    client,
    server,
  }
}
