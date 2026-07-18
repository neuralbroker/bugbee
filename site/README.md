# BugBee Landing Site

Premium marketing site for [BugBee](https://github.com/neuralbroker/bugbee) — NeuralBroker’s AI-powered bug detection platform.

## Stack

- **Next.js** (App Router, static export)
- **Tailwind CSS 3.4** + Framer Motion
- **React Three Fiber** (hero mesh gradient + particle fields)
- **Zustand**, React Hook Form + Zod
- **@vercel/analytics**

## Develop

```bash
cd site
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

## Build & deploy

```bash
npm run build   # outputs static site to site/out
```

Root [`vercel.json`](../vercel.json) builds this folder when the Vercel project root is the monorepo.

```bash
# from repo root
vercel --prod
```

## Structure

```
app/
  page.tsx              # Landing composition
  layout.tsx            # Fonts, analytics, providers
  sections/             # Nav → Footer (12 sections)
  components/ui/        # Button, Card, Accordion, …
  components/three/     # R3F hero + particles
  components/animations/
  hooks/ store/ lib/
```
