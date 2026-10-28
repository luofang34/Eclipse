# Eclipse Maps

Solar and lunar eclipse maps in the browser, for every eclipse from 2000 BCE to 2999, after the
maps of NASA's *Five Millennium Canons* (Espenak and Meeus). A solar eclipse shows its
penumbral limits, where it begins, ends and is greatest at sunrise and sunset, the curves of
magnitude 0.5, the path of totality or annularity, the shadow and the night side at any
moment, and the circumstances at any place clicked. A lunar eclipse shows the Moon's path
through the Earth's shadow and where on Earth the eclipse is seen.

- `crates/eclipse-core`: the eclipses, from the Sun and Moon of
  [Navigate](https://github.com/luofang34/Navigate)'s `navigate-celestial`, with Besselian
  geometry, the Canon's lunar radii and Danjon's shadow enlargement.
- `crates/eclipse-svg`: the maps as animated SVG files (SMIL), on an orthographic globe
  centred on greatest eclipse, or the lunar figure with the Moon moving through the shadow.
- `crates/eclipse-web`: the page, drawn on the globe of the
  [MapLibre fork](https://github.com/luofang34/maplibre-rs-experimental) with egui.

## Build and run

The page needs the `wasm32-unknown-unknown` target, [trunk](https://trunkrs.dev) and
`protoc` (the MapLibre fork compiles its tile schemas):

    cd crates/eclipse-web
    trunk serve            # or: trunk build --release

`?date=YYYY-MM-DD` opens the eclipse of that date. **Save SVG** in the time bar saves the
eclipse on show as an animated SVG.

Under **A place**, type a latitude and longitude or pick a city to see the eclipse from
there; **Find eclipses seen here** lists the next ones visible from it, from the year on show. **Stand at greatest
eclipse**, **Stand under the Moon** and **See the sky from this place** fly down to stand on
the ground and watch the Sun or the Moon, with Navigate's Sun, its Moon as seen from that
place, and its stars, drawn by the MapLibre fork's sky. Drag to look around and scroll to
change the field of view.

## Checks

    scripts/check.sh               # formatting, lints, tests, docs, the wasm build, structure
    scripts/check.sh long-range    # five millennia of the Canons and the ancient records
    scripts/check-pins.sh          # Navigate and the MapLibre fork at their main heads

CI runs the first two on every pull request and on main, and the third weekly.

The tests compare every eclipse of the Canons from 1800 to 2199, the Besselian elements and
paths NASA published for 2026, and totality as the expeditions recorded it at Königsberg in
1851 and at Sobral and Príncipe in 1919. With the `long-range` feature, which takes
Navigate's Sun and Moon from -2000 to 3000 fitted to JPL DE431, they compare every eclipse
of the Canons from -1999 to 2999; the records of Bur-Sagale (-762), Thales (-584), Babylon
(-135), Columbus (1504) and Halley (1715); the ranges of Delta T over which the untimed
total eclipses of Stephenson, Morrison and Hohenkerk (2016) put each place in the path; and
the contacts observers timed in 1851, 1878, 1900 and 1927. Dates before 1582 October 15
are Julian.
Their fixtures come from NASA's pages through `tools/canon_fixture.py`; the basemap's
coastlines and borders come from Natural Earth through `tools/natural_earth.py`.

## Licence

AGPL-3.0-or-later; see `LICENSE`.
