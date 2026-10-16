#!/usr/bin/env bash
# Fail when agent/user docs state facts that no longer match the code.
# Facts: CLI version (cli/Cargo.toml), MCP tool count (#[tool( in cli/src/commands/mcp.rs).
set -euo pipefail
cd "$(dirname "$0")/.."
version=$(sed -n 's/^version = "\(.*\)"/\1/p' cli/Cargo.toml | head -1)
tools=$(grep -c '#\[tool(' cli/src/commands/mcp.rs)
fail=0
check() { # file, regex-with-number, expected, label
  local f=$1 re=$2 want=$3 label=$4
  while IFS= read -r got; do
    [ "$got" = "$want" ] || { echo "$f: $label says $got, code says $want"; fail=1; }
  done < <(grep -oE "$re" "$f" | grep -oE '[0-9]+(\.[0-9]+)*' | head -20)
}
check AGENTS.md  'cli/`, v[0-9.]+'         "$version" "CLI version"
check CLAUDE.md  'CLI version:\*\* [0-9.]+' "$version" "CLI version"
for f in AGENTS.md README.md site/llms.txt; do
  check "$f" '([0-9]+ tools via|exposes [0-9]+ (MCP )?tools)' "$tools" "MCP tool count"
done
[ $fail = 0 ] && echo "doc facts ok (v$version, $tools MCP tools)"
exit $fail
