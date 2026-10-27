# Eclipse Maps

Solar and lunar eclipse maps in the browser, for every eclipse from 1850 to 2099, after the
maps of NASA's *Five Millennium Canons* (Espenak and Meeus). A solar eclipse shows its
penumbral limits, where it begins, ends and is greatest at sunrise and sunset, the curves of
magnitude 0.5, the path of totality or annularity, the shadow and the night side at any
moment, and the circumstances at any place clicked. A lunar eclipse shows the Moon's path
through the Earth's shadow and where on Earth the eclipse is seen.

- `crates/eclipse-core`: the eclipses, from the Sun and Moon of
  [Navigate](https://github.com/luofang34/Navigate)'s `navigate-celestial`, with Besselian
  geometry, the Canon's lunar radii and Danjon's shadow enlargement.
- `crates/eclipse-web`: the page, drawn on the globe of the
  [MapLibre fork](https://github.com/luofang34/maplibre-rs-experimental) with egui.

## Build and run

The page needs the `wasm32-unknown-unknown` target, [trunk](https://trunkrs.dev) and
`protoc` (the MapLibre fork compiles its tile schemas):

    cd crates/eclipse-web
    trunk serve            # or: trunk build --release

`?date=YYYY-MM-DD` opens the eclipse of that date.

## Checks

    cargo fmt --check
    cargo clippy -p eclipse-core --all-targets -- -D warnings
    cargo clippy -p eclipse-web --target wasm32-unknown-unknown --all-targets -- -D warnings
    cargo test -p eclipse-core --release
    RUSTDOCFLAGS="-D missing_docs -D broken_intra_doc_links" cargo doc -p eclipse-core --no-deps
    scripts/check-structure.sh

The tests compare every eclipse of the Canons from 1850 to 2099 and two published paths.
Their fixtures come from NASA's pages through `tools/canon_fixture.py`; the basemap's
coastlines and borders come from Natural Earth through `tools/natural_earth.py`.

## Licence

AGPL-3.0-or-later; see `LICENSE`.
