#!/usr/bin/env bun
/**
 * Publish any missing platform packages for a GitHub release version,
 * then ensure @neuralbroker/bugbee meta package lists all available platforms.
 *
 * Env:
 *   BUGBEE_VERSION   e.g. 1.0.0
 *   NODE_AUTH_TOKEN  npm token
 *   NPM_PUBLISH_DELAY_MS (default 15000)
 *   NPM_PUBLISH_RETRIES (default 10)
 */
import { $ } from "bun"
import path from "path"
import { fileURLToPath } from "url"

const version = (process.env.BUGBEE_VERSION ?? process.env.VERSION ?? "").replace(/^v/, "")
if (!version) {
  console.error("BUGBEE_VERSION is required (e.g. 1.0.0)")
  process.exit(1)
}
if (!process.env.NODE_AUTH_TOKEN) {
  console.error("NODE_AUTH_TOKEN is required")
  process.exit(1)
}

const delayMs = Number(process.env.NPM_PUBLISH_DELAY_MS ?? 15_000)
const maxAttempts = Number(process.env.NPM_PUBLISH_RETRIES ?? 10)
const root = path.resolve(fileURLToPath(new URL("../../..", import.meta.url)))
const work = path.join(root, "packages/bugbee/dist/remaining-publish")
const base = `https://github.com/neuralbroker/bugbee/releases/download/v${version}`
const metaName = "@neuralbroker/bugbee"

const platforms = [
  { name: "bugbee-linux-x64", archive: `${base}/bugbee-linux-x64.tar.gz`, binary: "bugbee", os: "linux", cpu: "x64" },
  {
    name: "bugbee-linux-x64-baseline",
    archive: `${base}/bugbee-linux-x64-baseline.tar.gz`,
    binary: "bugbee",
    os: "linux",
    cpu: "x64",
  },
  {
    name: "bugbee-linux-x64-musl",
    archive: `${base}/bugbee-linux-x64-musl.tar.gz`,
    binary: "bugbee",
    os: "linux",
    cpu: "x64",
  },
  {
    name: "bugbee-linux-x64-baseline-musl",
    archive: `${base}/bugbee-linux-x64-baseline-musl.tar.gz`,
    binary: "bugbee",
    os: "linux",
    cpu: "x64",
  },
  { name: "bugbee-linux-arm64", archive: `${base}/bugbee-linux-arm64.tar.gz`, binary: "bugbee", os: "linux", cpu: "arm64" },
  {
    name: "bugbee-linux-arm64-musl",
    archive: `${base}/bugbee-linux-arm64-musl.tar.gz`,
    binary: "bugbee",
    os: "linux",
    cpu: "arm64",
  },
  { name: "bugbee-darwin-x64", archive: `${base}/bugbee-darwin-x64.zip`, binary: "bugbee", os: "darwin", cpu: "x64" },
  {
    name: "bugbee-darwin-x64-baseline",
    archive: `${base}/bugbee-darwin-x64-baseline.zip`,
    binary: "bugbee",
    os: "darwin",
    cpu: "x64",
  },
  { name: "bugbee-darwin-arm64", archive: `${base}/bugbee-darwin-arm64.zip`, binary: "bugbee", os: "darwin", cpu: "arm64" },
  {
    name: "bugbee-windows-x64",
    archive: `${base}/bugbee-windows-x64.zip`,
    binary: "bugbee.exe",
    os: "win32",
    cpu: "x64",
  },
  {
    name: "bugbee-windows-x64-baseline",
    archive: `${base}/bugbee-windows-x64-baseline.zip`,
    binary: "bugbee.exe",
    os: "win32",
    cpu: "x64",
  },
  {
    name: "bugbee-windows-arm64",
    archive: `${base}/bugbee-windows-arm64.zip`,
    binary: "bugbee.exe",
    os: "win32",
    cpu: "arm64",
  },
] as const

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function published(name: string, ver = version) {
  return (await $`npm view ${name}@${ver} version`.nothrow()).exitCode === 0
}

async function npmPublish(dir: string, label: string) {
  await $`rm -f *.tgz`.cwd(dir).nothrow()
  await $`npm pack`.cwd(dir).quiet()
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const result = await $`npm publish *.tgz --access public --tag latest`.cwd(dir).nothrow()
    if (result.exitCode === 0) {
      console.log(`published ${label}`)
      await sleep(delayMs)
      return
    }
    const stderr = result.stderr.toString("utf8")
    const retriable = /E429|Too Many Requests|ETIMEDOUT|ECONNRESET|network|socket/i.test(stderr)
    if (!retriable || attempt === maxAttempts) {
      console.error(stderr)
      throw new Error(`npm publish failed for ${label}`)
    }
    const wait = delayMs * attempt
    console.log(`retry ${attempt}/${maxAttempts} for ${label} in ${wait}ms`)
    await sleep(wait)
  }
}

async function publishPlatform(p: (typeof platforms)[number]) {
  if (await published(p.name)) {
    console.log(`already on npm: ${p.name}@${version}`)
    return
  }

  const dir = path.join(work, p.name)
  await $`rm -rf ${dir}`
  await $`mkdir -p ${path.join(dir, "bin")}`
  const asset = path.join(work, `${p.name}.asset`)
  console.log(`downloading ${p.archive}`)
  await $`curl -fsSL -o ${asset} ${p.archive}`
  if (p.archive.endsWith(".zip")) {
    await $`unzip -q ${asset} -d ${path.join(dir, "bin")}`
  } else {
    await $`tar -xzf ${asset} -C ${path.join(dir, "bin")}`
  }

  const binDir = path.join(dir, "bin")
  const wanted = path.join(binDir, p.binary)
  const plain = path.join(binDir, "bugbee")
  if (p.binary === "bugbee.exe" && (await Bun.file(plain).exists()) && !(await Bun.file(wanted).exists())) {
    await $`mv ${plain} ${wanted}`
  }
  if (!(await Bun.file(wanted).exists())) {
    await $`ls -la ${binDir}`
    throw new Error(`missing binary ${wanted}`)
  }
  await $`chmod 755 ${wanted}`.nothrow()

  await Bun.file(path.join(dir, "package.json")).write(
    JSON.stringify(
      {
        name: p.name,
        version,
        preferUnplugged: true,
        os: [p.os],
        cpu: [p.cpu],
      },
      null,
      2,
    ) + "\n",
  )
  await npmPublish(dir, `${p.name}@${version}`)
}

async function listAvailable() {
  const deps: Record<string, string> = {}
  for (const p of platforms) {
    if (await published(p.name)) deps[p.name] = version
  }
  return deps
}

async function ensureMeta(deps: Record<string, string>) {
  if (Object.keys(deps).length === 0) throw new Error("no platform packages on npm")

  let metaVersion = version
  if (await published(metaName, version)) {
    const raw = await $`npm view ${metaName}@${version} optionalDependencies --json`.nothrow()
    const got = raw.exitCode === 0 ? (JSON.parse(raw.stdout.toString("utf8") || "{}") as Record<string, string>) : {}
    const missing = Object.keys(deps).filter((k) => !got[k])
    if (missing.length === 0) {
      console.log(`meta ${metaName}@${version} already complete`)
      return
    }
    const parts = version.split(".")
    parts[parts.length - 1] = String(Number(parts[parts.length - 1]) + 1)
    metaVersion = parts.join(".")
    console.log(`meta incomplete (missing ${missing.join(", ")}); publishing ${metaName}@${metaVersion}`)
  }

  const dir = path.join(work, "meta")
  await $`rm -rf ${dir}`
  await $`mkdir -p ${path.join(dir, "bin")}`
  await $`cp ${path.join(root, "packages/bugbee/script/postinstall.mjs")} ${path.join(dir, "postinstall.mjs")}`
  await $`cp ${path.join(root, "LICENSE")} ${path.join(dir, "LICENSE")}`
  await Bun.file(path.join(dir, "bin/bugbee.exe")).write(
    [
      "#!/bin/sh",
      'echo "Error: bugbee postinstall did not run. Reinstall with: npm install -g @neuralbroker/bugbee" >&2',
      "exit 1",
      "",
    ].join("\n"),
  )
  await $`chmod 755 ${path.join(dir, "bin/bugbee.exe")}`
  await Bun.file(path.join(dir, "package.json")).write(
    JSON.stringify(
      {
        name: metaName,
        description: "Bugbee — open source AI coding agent",
        repository: { type: "git", url: "https://github.com/neuralbroker/bugbee" },
        homepage: "https://github.com/neuralbroker/bugbee",
        bin: { bugbee: "./bin/bugbee.exe" },
        scripts: { postinstall: "node ./postinstall.mjs" },
        version: metaVersion,
        license: "MIT",
        os: ["darwin", "linux", "win32"],
        cpu: ["arm64", "x64"],
        optionalDependencies: deps,
      },
      null,
      2,
    ) + "\n",
  )
  await npmPublish(dir, `${metaName}@${metaVersion}`)
}

await $`rm -rf ${work}`
await $`mkdir -p ${work}`
console.log("publish-remaining", { version, delayMs, maxAttempts })

const failures: string[] = []
for (const p of platforms) {
  try {
    await publishPlatform(p)
  } catch (error) {
    console.error(error)
    failures.push(p.name)
  }
}

const deps = await listAvailable()
console.log("available platforms", Object.keys(deps).sort())
await ensureMeta(deps)

console.log("\n=== inventory ===")
for (const name of [metaName, ...platforms.map((p) => p.name)]) {
  const ok = await published(name, name === metaName ? undefined : version)
  // meta may be on a bumped version
  if (name === metaName) {
    const latest = await $`npm view ${metaName} version`.nothrow()
    console.log(`${name}: ${latest.exitCode === 0 ? latest.stdout.toString("utf8").trim() : "MISSING"}`)
  } else {
    console.log(`${name}: ${ok ? version : "MISSING"}`)
  }
}

if (failures.length) {
  console.error("platform publish failures:", failures.join(", "))
  process.exit(1)
}
console.log("publish-remaining done")
