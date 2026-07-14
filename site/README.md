# Bugbee product site

The private-beta product site is a statically exported Next.js application.
It uses CSS motion and a compressed local WebP hero asset; no public source,
release, or analytics integration is rendered on the page.

Deploy with the repository root as the Vercel project root. The root
[`vercel.json`](../vercel.json) installs, builds, and deploys `site/out`.

## Local preview

```bash
cd site
npm ci
npm run dev
# open http://localhost:3000
```

## Deploy

```bash
# Vercel dashboard: import the repository with Root Directory = .
```

Not affiliated with SpaceXAI; aesthetic inspiration only.
