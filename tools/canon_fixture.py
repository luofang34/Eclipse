"""Writes eclipse-core's test fixtures from NASA's eclipse pages (public domain).

Usage:
    python3 -I tools/canon_fixture.py CAT_DIR crates/eclipse-core/tests/fixtures \
        2026_08_12=SE2026Aug12Tpath.html 2026_02_17=SE2026Feb17Apath.html

CAT_DIR holds the Five Millennium Canon catalogue pages SE1801-1900.html,
SE1901-2000.html, SE2001-2100.html, LE1801-1900.html, LE1901-2000.html and
LE2001-2100.html from https://eclipse.gsfc.nasa.gov/SEcat5/ and /LEcat5/.
Each NAME=PATH_HTML is a path table from https://eclipse.gsfc.nasa.gov/SEpath/SEpath2001/,
written to path_NAME.csv; a limit the table leaves out ("-") is left empty.
Rows from 1850 to 2099 are kept: Navigate's Sun and Moon end at 2100 January 1.
"""
import csv
import html
import re
import sys
from pathlib import Path

MONTHS = "Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec".split()
DATE = r"(\d{4}) (\w{3}) (\d{2})\s+(\d{2}:\d{2}:\d{2})"


def text(path):
    raw = path.read_bytes().decode("latin-1")
    return html.unescape(re.sub(r"<[^>]*>", "", raw))


def angle(value, positive):
    """'37N' -> 37, '88W' -> -88."""
    number, hemisphere = float(value[:-1]), value[-1]
    return number if hemisphere == positive else -number


def minutes_seconds(value):
    m = re.fullmatch(r"(\d+)m(\d+)s", value)
    return str(int(m.group(1)) * 60 + int(m.group(2)))


def iso(year, month, day):
    return f"{year}-{MONTHS.index(month) + 1:02d}-{day}"


def solar(cat_dir):
    pattern = re.compile(
        r"^\s*\d{5}\s+" + DATE + r"\s+(-?\d+)\s+(-?\d+)\s+(\d+)\s+(\w)\w?[+-]?\s+\S+\s+"
        r"(-?\d\.\d{4})\s+(\d\.\d{4})\s+(\d+[NS])\s+(\d+[EW])\s+(\d+)(?:\s+(\S+)\s+(\S+))?\s*$"
    )
    rows = []
    for name in ("SE1801-1900", "SE1901-2000", "SE2001-2100"):
        for line in text(cat_dir / f"{name}.html").splitlines():
            m = pattern.match(line)
            if not m or not 1850 <= int(m.group(1)) <= 2099:
                continue
            y, mo, d, td, dt, luna, saros, kind, gamma, mag, lat, lon, alt, width, dur = m.groups()
            rows.append([iso(y, mo, d), td, luna, saros, kind, gamma, mag,
                         angle(lat, "N"), angle(lon, "E"), alt,
                         "" if width in (None, "-") else width,
                         minutes_seconds(dur) if dur not in (None, "-") else "", dt])
    return ["date", "td", "lunation", "saros", "type", "gamma", "magnitude",
            "lat", "lon", "sun_alt", "width_km", "duration_s", "delta_t"], rows


def lunar(cat_dir):
    pattern = re.compile(
        r"^\s*\d{5}\s+" + DATE + r"\s+(-?\d+)\s+(-?\d+)\s+(\d+)\s+(\w)\w?[+-]?\s+\S+\s+"
        r"(-?\d\.\d{4})\s+(-?\d\.\d{4})\s+(-?\d\.\d{4})\s+(\S+)\s+(\S+)\s+(\S+)\s+"
    )
    rows = []
    for name in ("LE1801-1900", "LE1901-2000", "LE2001-2100"):
        for line in text(cat_dir / f"{name}.html").splitlines():
            m = pattern.match(line)
            if not m or not 1850 <= int(m.group(1)) <= 2099:
                continue
            y, mo, d, td, dt, luna, saros, kind, gamma, pen, um, pdur, udur, tdur = m.groups()
            rows.append([iso(y, mo, d), td, luna, saros, kind, gamma, pen, um,
                         pdur, "" if udur == "-" else udur, "" if tdur == "-" else tdur])
    return ["date", "td", "lunation", "saros", "type", "gamma", "pen_mag", "um_mag",
            "pen_min", "par_min", "total_min"], rows


def path(path_html):
    point = r"(?:(\d+) (\d+\.\d)([NS])\s+(\d+) (\d+\.\d)([EW])|-\s+-)"
    pattern = re.compile(r"^\s*(\d{2}:\d{2})\s+" + point + r"\s+" + point + r"\s+" + point)
    rows = []
    for line in text(path_html).splitlines():
        m = pattern.match(line)
        if not m:
            continue
        g = m.groups()
        values = [g[0]]
        for i in range(1, 19, 6):
            if g[i] is None:
                values += ["", ""]
                continue
            lat = int(g[i]) + float(g[i + 1]) / 60
            lon = int(g[i + 3]) + float(g[i + 4]) / 60
            values += [round(lat if g[i + 2] == "N" else -lat, 4),
                       round(lon if g[i + 5] == "E" else -lon, 4)]
        rows.append(values)
    return ["ut", "north_lat", "north_lon", "south_lat", "south_lon",
            "central_lat", "central_lon"], rows


def write(target, header_rows):
    header, rows = header_rows
    with target.open("w", newline="") as f:
        out = csv.writer(f, lineterminator="\n")
        out.writerow(header)
        out.writerows(rows)
    print(f"{target}: {len(rows)} rows")


def main():
    cat_dir, out = Path(sys.argv[1]), Path(sys.argv[2])
    write(out / "solar_catalogue.csv", solar(cat_dir))
    write(out / "lunar_catalogue.csv", lunar(cat_dir))
    for pair in sys.argv[3:]:
        name, path_html = pair.split("=", 1)
        write(out / f"path_{name}.csv", path(Path(path_html)))


main()
