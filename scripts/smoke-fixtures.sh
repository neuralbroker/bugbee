#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/release/bugbee}"
test -x "$binary"

workspace="$(mktemp -d)"
trap 'rm -rf "$workspace"' EXIT

for fixture in fixtures/python-vuln fixtures/js-vuln fixtures/go-vuln fixtures/india-portal; do
  name="$(basename "$fixture")"
  root="$workspace/$name"
  mkdir -p "$root"
  cp -R "$fixture/." "$root/"

  "$binary" --root "$root" init >/dev/null
  "$binary" --root "$root" hunt >"$root/hunt.out"
  "$binary" --root "$root" findings >"$root/findings.out"
  "$binary" --root "$root" report --output "$root/findings.sarif.json" >/dev/null

  python3 -c '
import json, sys
report = json.load(open(sys.argv[1]))
assert report["version"] == "2.1.0" and report["runs"], "invalid SARIF report"
' "$root/findings.sarif.json"
  grep -Eq 'findings[[:space:]]+:[[:space:]]+[1-9][0-9]*' "$root/hunt.out"
  printf 'smoke passed: %s\n' "$name"
done
