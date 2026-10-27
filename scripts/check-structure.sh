#!/usr/bin/env bash
# Structural rules the Rust gates do not enforce: file size, module layout and names, and
# lib.rs holding only declarations and re-exports under a crate doc comment.
set -euo pipefail
cd "$(dirname "$0")/.."
status=0
while IFS= read -r file; do
    lines=$(wc -l < "$file")
    if [ "$lines" -gt 500 ]; then
        echo "$file: $lines lines; split it below 500" >&2
        status=1
    fi
    case "$(basename "$file")" in
        mod.rs) echo "$file: use foo.rs with foo/, not mod.rs" >&2; status=1 ;;
        utils.rs | helpers.rs | common.rs) echo "$file: name modules by domain" >&2; status=1 ;;
    esac
done < <(find crates -name '*.rs' -not -path '*/target/*')
for lib in crates/*/src/lib.rs; do
    [ -e "$lib" ] || continue
    if [ "$(wc -l < "$lib")" -gt 100 ]; then
        echo "$lib: over 100 lines" >&2
        status=1
    fi
    if ! head -1 "$lib" | grep -q '^//!'; then
        echo "$lib: must start with a //! crate doc comment" >&2
        status=1
    fi
    if grep -n -E '^\s*(pub(\(crate\))? )?(fn|struct|enum|impl|const|static|trait|type) ' "$lib" >&2; then
        echo "$lib: only module declarations and re-exports belong here" >&2
        status=1
    fi
done
exit "$status"
