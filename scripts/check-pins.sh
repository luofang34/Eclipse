#!/usr/bin/env bash
# Whether Navigate and the MapLibre fork are pinned at the heads of their main branches; names
# each pin that is behind. Network access to GitHub is needed.
set -euo pipefail
cd "$(dirname "$0")/.."
status=0
check() {
    local repo=$1 manifest=$2
    local pinned head
    pinned=$(grep -o "$repo\", rev = \"[0-9a-f]*" "$manifest" | sed 's/.*rev = "//')
    head=$(git ls-remote "$repo" refs/heads/main | cut -f1)
    if [ -z "$pinned" ] || [ -z "$head" ]; then
        echo "$repo: pin or head not found" >&2
        status=1
    elif [ "$pinned" != "$head" ]; then
        echo "$repo: pinned at $pinned in $manifest, main is at $head" >&2
        status=1
    else
        echo "$repo: at main $head"
    fi
}
check https://github.com/luofang34/Navigate Cargo.toml
check https://github.com/luofang34/maplibre-rs-experimental crates/eclipse-web/Cargo.toml
exit $status
