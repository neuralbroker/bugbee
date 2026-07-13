# Bugbee product site (Vercel)

SpaceXAI/x.ai–inspired dark product landing for Bugbee.

Static HTML/CSS. The checked-in Vercel configuration supports either setup:

- Preferred: use the repository root; root [`vercel.json`](../vercel.json)
  deploys this directory as the output.
- Existing project setup: use **Root Directory = `site`**; this directory's
  [`vercel.json`](./vercel.json) applies the same security headers.

## Local preview

```bash
cd site
python3 -m http.server 4173
# open http://127.0.0.1:4173
```

## Deploy

```bash
# Vercel dashboard: import repo, Root Directory = site
# or
cd site && npx vercel
```

Not affiliated with SpaceXAI; aesthetic inspiration only.
