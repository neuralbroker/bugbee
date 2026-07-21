<p align="center">
  <a href="https://github.com/neuralbroker/bugbee">
    <picture>
      <source srcset="packages/console/app/src/asset/logo-ornate-dark.svg" media="(prefers-color-scheme: dark)">
      <source srcset="packages/console/app/src/asset/logo-ornate-light.svg" media="(prefers-color-scheme: light)">
      <img src="packages/console/app/src/asset/logo-ornate-light.svg" alt="Bugbee logo">
    </picture>
  </a>
</p>
<p align="center">AI-kodeagent med åpen kildekode.</p>
<p align="center">
  <a href="https://github.com/neuralbroker/bugbee/discord"><img alt="Discord" src="https://img.shields.io/discord/1391832426048651334?style=flat-square&label=discord" /></a>
  <a href="https://www.npmjs.com/package/bugbee-ai"><img alt="npm" src="https://img.shields.io/npm/v/bugbee-ai?style=flat-square" /></a>
  <a href="https://github.com/neuralbroker/bugbee/actions/workflows/publish.yml"><img alt="Build status" src="https://img.shields.io/github/actions/workflow/status/neuralbroker/bugbee/publish.yml?style=flat-square&branch=dev" /></a>
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.zh.md">简体中文</a> |
  <a href="README.zht.md">繁體中文</a> |
  <a href="README.ko.md">한국어</a> |
  <a href="README.de.md">Deutsch</a> |
  <a href="README.es.md">Español</a> |
  <a href="README.fr.md">Français</a> |
  <a href="README.it.md">Italiano</a> |
  <a href="README.da.md">Dansk</a> |
  <a href="README.ja.md">日本語</a> |
  <a href="README.pl.md">Polski</a> |
  <a href="README.ru.md">Русский</a> |
  <a href="README.bs.md">Bosanski</a> |
  <a href="README.ar.md">العربية</a> |
  <a href="README.no.md">Norsk</a> |
  <a href="README.br.md">Português (Brasil)</a> |
  <a href="README.th.md">ไทย</a> |
  <a href="README.tr.md">Türkçe</a> |
  <a href="README.uk.md">Українська</a> |
  <a href="README.bn.md">বাংলা</a> |
  <a href="README.gr.md">Ελληνικά</a> |
  <a href="README.vi.md">Tiếng Việt</a>
</p>

[![Bugbee Terminal UI](packages/web/src/assets/lander/screenshot.png)](https://github.com/neuralbroker/bugbee)

---

### Installasjon

```bash
# YOLO
curl -fsSL https://github.com/neuralbroker/bugbee/install | bash

# Pakkehåndterere
npm i -g bugbee-ai@latest        # eller bun/pnpm/yarn
scoop install bugbee             # Windows
choco install bugbee             # Windows
brew install neuralbroker/tap/bugbee # macOS og Linux (anbefalt, alltid oppdatert)
brew install bugbee              # macOS og Linux (offisiell brew-formel, oppdateres sjeldnere)
sudo pacman -S bugbee            # Arch Linux (Stable)
paru -S bugbee-bin               # Arch Linux (Latest from AUR)
mise use -g bugbee               # alle OS
nix run nixpkgs#bugbee           # eller github:neuralbroker/bugbee for nyeste dev-branch
```

> [!TIP]
> Fjern versjoner eldre enn 0.1.x før du installerer.

### Desktop-app (BETA)

Bugbee er også tilgjengelig som en desktop-app. Last ned direkte fra [releases-siden](https://github.com/neuralbroker/bugbee/releases) eller [bugbee.dev/download](https://github.com/neuralbroker/bugbee/download).

| Plattform             | Nedlasting                         |
| --------------------- | ---------------------------------- |
| macOS (Apple Silicon) | `bugbee-desktop-mac-arm64.dmg`   |
| macOS (Intel)         | `bugbee-desktop-mac-x64.dmg`     |
| Windows               | `bugbee-desktop-windows-x64.exe` |
| Linux                 | `.deb`, `.rpm` eller AppImage      |

```bash
# macOS (Homebrew)
brew install --cask bugbee-desktop
# Windows (Scoop)
scoop bucket add extras; scoop install extras/bugbee-desktop
```

#### Installasjonsmappe

Installasjonsskriptet bruker følgende prioritet for installasjonsstien:

1. `$BUGBEE_INSTALL_DIR` - Egendefinert installasjonsmappe
2. `$XDG_BIN_DIR` - Sti som følger XDG Base Directory Specification
3. `$HOME/bin` - Standard brukerbinar-mappe (hvis den finnes eller kan opprettes)
4. `$HOME/.bugbee/bin` - Standard fallback

```bash
# Eksempler
BUGBEE_INSTALL_DIR=/usr/local/bin curl -fsSL https://github.com/neuralbroker/bugbee/install | bash
XDG_BIN_DIR=$HOME/.local/bin curl -fsSL https://github.com/neuralbroker/bugbee/install | bash
```

### Agents

Bugbee har to innebygde agents du kan bytte mellom med `Tab`-tasten.

- **build** - Standard, agent med full tilgang for utviklingsarbeid
- **plan** - Skrivebeskyttet agent for analyse og kodeutforsking
  - Nekter filendringer som standard
  - Spør om tillatelse før bash-kommandoer
  - Ideell for å utforske ukjente kodebaser eller planlegge endringer

Det finnes også en **general**-subagent for komplekse søk og flertrinnsoppgaver.
Den brukes internt og kan kalles via `@general` i meldinger.

Les mer om [agents](https://github.com/neuralbroker/bugbee/docs/agents).

### Dokumentasjon

For mer info om hvordan du konfigurerer Bugbee, [**se dokumentasjonen**](https://github.com/neuralbroker/bugbee/docs).

### Bidra

Hvis du vil bidra til Bugbee, les [contributing docs](./CONTRIBUTING.md) før du sender en pull request.

### Bygge på Bugbee

Hvis du jobber med et prosjekt som er relatert til Bugbee og bruker "bugbee" som en del av navnet; for eksempel "bugbee-dashboard" eller "bugbee-mobile", legg inn en merknad i README som presiserer at det ikke er bygget av Bugbee-teamet og ikke er tilknyttet oss på noen måte.

---

**Bli med i fellesskapet** [Discord](https://discord.gg/bugbee) | [X.com](https://x.com/bugbee)
