const stage = process.env.SST_STAGE || "dev"

export default {
  url: stage === "production" ? "https://github.com/neuralbroker/bugbee" : `https://${stage}.bugbee.ai`,
  console: stage === "production" ? "https://github.com/neuralbroker/bugbee/auth" : `https://${stage}.bugbee.ai/auth`,
  email: "help@anoma.ly",
  socialCard: "https://social-cards.sst.dev",
  github: "https://github.com/neuralbroker/bugbee",
  discord: "https://github.com/neuralbroker/bugbee/discord",
  headerLinks: [
    { name: "app.header.home", url: "/" },
    { name: "app.header.docs", url: "/docs/" },
  ],
}
