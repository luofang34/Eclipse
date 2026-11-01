#!/usr/bin/env bash
# Builds the site: the page into crates/eclipse-web/dist and the book into its book/, where the
# book's frames find the page one level up. The argument is the path the site is served from.
set -euo pipefail
cd "$(dirname "$0")/.."
public_url=${1:-/}
(cd crates/eclipse-web && trunk build --release --locked --public-url "$public_url")
mdbook build book
\rm -rf crates/eclipse-web/dist/book
cp -R book/build crates/eclipse-web/dist/book
