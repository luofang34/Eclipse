"""Writes the basemap's GeoJSON from Natural Earth (public domain).

Usage: python3 -I natural_earth.py NE_DIR data

NE_DIR holds ne_50m_land.geojson and ne_50m_admin_0_boundary_lines_land.geojson
from github.com/nvkelso/natural-earth-vector. Properties are dropped and
coordinates rounded to 0.02 degrees, about 2 km, finer than a globe-scale pixel.

coast.geojson is the land's outline without the edges no shore makes: where a polygon
is cut along the antimeridian, and where Antarctica's is closed across the pole.

borders.geojson holds the boundaries Ukraine's worldview in Natural Earth (its FCLASS_UA
field) draws between countries. It leaves out the line across the Perekop isthmus between
Crimea and the mainland, and those around Somaliland, Western Sahara and Kosovo, which
it does not recognise as borders.
"""
import json
import sys
from pathlib import Path

STEP = 0.02


def ring(points):
    out = []
    for lon, lat, *_ in points:
        p = [round(round(lon / STEP) * STEP, 2), round(round(lat / STEP) * STEP, 2)]
        if not out or out[-1] != p:
            out.append(p)
    return out


def geometry(g):
    kind, coords = g["type"], g["coordinates"]
    if kind == "LineString":
        return {"type": kind, "coordinates": ring(coords)}
    if kind == "MultiLineString":
        return {"type": kind, "coordinates": [ring(c) for c in coords]}
    if kind == "Polygon":
        rings = [r for r in (ring(c) for c in coords) if len(r) >= 4]
        return {"type": kind, "coordinates": rings} if rings else None
    if kind == "MultiPolygon":
        polys = [[r for r in (ring(c) for c in p) if len(r) >= 4] for p in coords]
        polys = [p for p in polys if p]
        return {"type": kind, "coordinates": polys} if polys else None
    raise ValueError(f"unexpected geometry {kind}")


# A boundary line's class in Ukraine's worldview, where it differs from the default.
WORLDVIEW = "FCLASS_UA"
# The classes of a line that is not a boundary between countries in that worldview.
NOT_BETWEEN_COUNTRIES = {"Admin-1 boundary", "Unrecognized"}


def between_countries(properties):
    """Whether the worldview draws this boundary line between two countries."""
    kind = properties.get(WORLDVIEW) or properties["FEATURECLA"]
    return kind not in NOT_BETWEEN_COUNTRIES


def convert(source, target, keep=lambda properties: True):
    features = []
    for f in json.loads(source.read_text())["features"]:
        if not keep(f["properties"]):
            continue
        g = geometry(f["geometry"])
        if g:
            features.append({"type": "Feature", "properties": {}, "geometry": g})
    doc = {"type": "FeatureCollection", "features": features}
    target.write_text(json.dumps(doc, separators=(",", ":")))
    print(f"{target}: {len(features)} features, {target.stat().st_size} bytes")


def artificial(a, b):
    """Whether the edge from a to b is a cut rather than a shore."""
    along_antimeridian = abs(a[0]) == 180 and abs(b[0]) == 180
    across_the_pole = a[1] < -85 and b[1] < -85
    return along_antimeridian or across_the_pole


def coast(source, target):
    lines = []
    for f in json.loads(source.read_text())["features"]:
        g = geometry(f["geometry"])
        if not g:
            continue
        polygons = g["coordinates"] if g["type"] == "MultiPolygon" else [g["coordinates"]]
        for polygon in polygons:
            for r in polygon:
                line = [r[0]]
                for a, b in zip(r, r[1:]):
                    if artificial(a, b):
                        if len(line) > 1:
                            lines.append(line)
                        line = [b]
                    else:
                        line.append(b)
                if len(line) > 1:
                    lines.append(line)
    doc = {"type": "FeatureCollection", "features": [
        {"type": "Feature", "properties": {},
         "geometry": {"type": "MultiLineString", "coordinates": lines}}]}
    target.write_text(json.dumps(doc, separators=(",", ":")))
    print(f"{target}: {len(lines)} lines, {target.stat().st_size} bytes")


def main():
    src, dst = Path(sys.argv[1]), Path(sys.argv[2])
    convert(src / "ne_50m_land.geojson", dst / "land.geojson")
    coast(src / "ne_50m_land.geojson", dst / "coast.geojson")
    convert(
        src / "ne_50m_admin_0_boundary_lines_land.geojson",
        dst / "borders.geojson",
        between_countries,
    )


main()
