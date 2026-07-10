# Releasing Bugbee

The release workflow publishes native CLI archives for Linux x86_64, macOS
Intel, macOS Apple Silicon, and Windows x86_64 whenever a version tag is
pushed.

Before tagging a release, ensure the pull request CI is green and update the
version in the workspace manifest. Then create and push an annotated tag:

```bash
git checkout main
git pull --ff-only
git tag -a v0.1.0 -m "Bugbee v0.1.0"
git push origin v0.1.0
```

GitHub Actions creates the release and attaches platform-specific archives.
