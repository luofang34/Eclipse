#!/usr/bin/env bash
# The local gate, in the order CI runs it. With no argument: formatting, lints for the native,
# wasm32 and long-range builds, the tests, docs, the release wasm build and the structure
# rules. With `long-range`: eclipse-core's tests with Navigate's -2000..3000 Sun and Moon,
# which compare every eclipse of NASA's Five Millennium Canons from -1999 to 2999 and the
# ancient records, and take a few minutes.
set -euo pipefail
cd "$(dirname "$0")/.."

step() {
    echo "== $*"
    "$@"
}

if [ "${1:-}" = "long-range" ]; then
    step cargo test -p eclipse-core --release --locked --features long-range
    exit 0
fi

step cargo fmt --check
step cargo clippy --locked -p eclipse-core -p eclipse-svg -p eclipse-web --all-targets -- -D warnings
step cargo clippy --locked -p eclipse-core --all-targets --features long-range -- -D warnings
step cargo clippy --locked -p eclipse-web --target wasm32-unknown-unknown --all-targets -- -D warnings
step cargo test --locked -p eclipse-core -p eclipse-svg -p eclipse-web --release
RUSTDOCFLAGS="-D missing_docs -D broken_intra_doc_links" \
    step cargo doc --locked -p eclipse-core -p eclipse-svg --no-deps
step cargo build --locked -p eclipse-web --target wasm32-unknown-unknown --release
step scripts/check-structure.sh
