const stage = process.env.SST_STAGE || "dev"

export default {
  url: stage === "production" ? "https://bugbee.dev" : `https://${stage}.bugbee.ai`,
  console: stage === "production" ? "https://bugbee.dev/auth" : `https://${stage}.bugbee.ai/auth`,
  email: "help@anoma.ly",
  socialCard: "https://social-cards.sst.dev",
  github: "https://github.com/neuralbroker/bugbee",
  discord: "https://bugbee.dev/discord",
  headerLinks: [
    { name: "app.header.home", url: "/" },
    { name: "app.header.docs", url: "/docs/" },
  ],
}
