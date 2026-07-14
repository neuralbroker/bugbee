# Bugbee product site (Next.js + Vercel)

Cinematic, motion-driven Next.js landing page for Bugbee.

The checked-in Vercel configuration supports either setup:

- Preferred: use the repository root; root [`vercel.json`](../vercel.json)
  deploys this directory as the output.
- Existing project setup: use **Root Directory = `site`**; this directory's
  [`vercel.json`](./vercel.json) applies the same security headers.

## Local preview

```bash
cd site
npm install
npm run dev
# open http://localhost:3000
```

## Deploy

```bash
# Vercel dashboard: import repo, Root Directory = site
# or
cd site && npx vercel
```

Not affiliated with SpaceXAI; aesthetic inspiration only.
