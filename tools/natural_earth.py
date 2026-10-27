"""Writes the basemap's GeoJSON from Natural Earth (public domain).

Usage: python3 -I tools/natural_earth.py NE_DIR crates/eclipse-web/assets

NE_DIR holds ne_50m_land.geojson and ne_50m_admin_0_boundary_lines_land.geojson
from github.com/nvkelso/natural-earth-vector. Properties are dropped and
coordinates rounded to 0.02 degrees, about 2 km, finer than a globe-scale pixel.
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


def convert(source, target):
    features = []
    for f in json.loads(source.read_text())["features"]:
        g = geometry(f["geometry"])
        if g:
            features.append({"type": "Feature", "properties": {}, "geometry": g})
    doc = {"type": "FeatureCollection", "features": features}
    target.write_text(json.dumps(doc, separators=(",", ":")))
    print(f"{target}: {len(features)} features, {target.stat().st_size} bytes")


def main():
    src, dst = Path(sys.argv[1]), Path(sys.argv[2])
    convert(src / "ne_50m_land.geojson", dst / "land.geojson")
    convert(src / "ne_50m_admin_0_boundary_lines_land.geojson", dst / "borders.geojson")


main()
