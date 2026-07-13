# Releasing Bugbee

The release workflow publishes native CLI archives for Linux x86_64, macOS
Intel, macOS Apple Silicon, and Windows x86_64 whenever a version tag is
pushed. Users install with:

```bash
curl -fsSL https://github.com/neuralbroker/bugbee/releases/latest/download/install-bugbee.sh | bash
```

Before tagging a release, ensure the pull request CI is green and update the
version in the workspace manifest. Then create and push an annotated tag:

```bash
git checkout main
git pull --ff-only
git tag -a v0.1.0-beta.2 -m "Bugbee v0.1.0-beta.2"
git push origin v0.1.0-beta.2
```

GitHub Actions creates the release and attaches platform-specific archives.
Hyphenated tags (e.g. `beta`) are published as GitHub prereleases.

After the first release lands, smoke-test the installer:

```bash
# attach the installer to the release (required for the one-liner)
cp scripts/install.sh /tmp/install-bugbee.sh
gh release upload "v0.1.0-beta.2" /tmp/install-bugbee.sh --clobber

curl -fsSL https://github.com/neuralbroker/bugbee/releases/latest/download/install-bugbee.sh | bash
bugbee --version
```
