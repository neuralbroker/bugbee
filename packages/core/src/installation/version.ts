declare global {
  const BUGBEE_VERSION: string
  const BUGBEE_CHANNEL: string
}

export const InstallationVersion = typeof BUGBEE_VERSION === "string" ? BUGBEE_VERSION : "local"
export const InstallationChannel = typeof BUGBEE_CHANNEL === "string" ? BUGBEE_CHANNEL : "local"
export const InstallationLocal = InstallationChannel === "local"
