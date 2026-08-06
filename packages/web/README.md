# Bugbee documentation site

This package contains the public Bugbee documentation and product landing page. It is an
Astro/Starlight site with localized content, installation guidance, API documentation,
provider setup, plugin references, and shareable session views.

## Project structure

```text
src/
├── components/       # Landing page, navigation, and share views
├── content/docs/     # Localized documentation
├── content/i18n/     # Product and UI translations
└── pages/            # Astro routes
```

## Commands

Run from this package directory:

```bash
bun run dev
bun run build
bun run preview
```

The site is part of the root workspace. Prefer running commands from the repository root
when working across documentation and application packages.
