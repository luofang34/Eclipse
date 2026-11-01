# Against the Canons

Espenak and Meeus's *Five Millennium Canons* cover -1999 to 3000: 11,898 solar eclipses and
12,064 lunar ones. Each of them is a row in this project's test data, and each row is recomputed
and compared on every change, by the `long-range` checks in continuous integration.

The tests are the claim. The figures below are the limits they enforce.

## Solar eclipses

| Quantity | Allowed difference from the Canon |
| -------- | --------------------------------- |
| Type, lunation, Saros | None, save the borderline cases below |
| Gamma | 0.0015 |
| Magnitude, central eclipses | 0.0015 |
| Magnitude, partial eclipses | 0.003 |
| Latitude, longitude and Sun's altitude at greatest | 0.75° of the Canon's whole degrees, or 1.5° with the Sun under 10° |
| Central duration | 4 seconds |
| Path width | 3 km, or 3%, or 2% for each 1000 km of the path's footprint, whichever is most |

Longitude also allows for the turn of the Earth between this project's ΔT and the Canon's, and
for a degree of longitude being short near the poles.

## Lunar eclipses

| Quantity | Allowed difference from the Canon |
| -------- | --------------------------------- |
| Type, lunation, Saros | None, save the borderline cases below |
| Gamma | 0.0015 |
| Penumbral and umbral magnitudes | 0.003 |
| Duration of a phase of D minutes | 0.5 + 60 / D minutes |

The last row is loose for short phases because a phase that barely happens is a short chord
across the edge of the shadow, and its length changes quickly with the Moon's distance from
that edge.

## Where disagreement is allowed

Three kinds of eclipse may differ in type, because the answer really is on a knife edge:

- A solar eclipse whose umbra grazes the Earth's limb, where whether it touches at all comes
  down to kilometres.
- A solar eclipse whose central line ends with the umbra's radius at about zero, within the
  elements' accuracy, which can read as total, annular or hybrid.
- A lunar eclipse whose magnitude is within 0.003 of a threshold.

The magnitude of a grazing eclipse is not compared, nor are path widths past gamma 0.95 or
under 5 km. Near the limb the Canon's
width and the distance between the limits part by up to a third, as they measure different
things under a very low Sun.

## The paths of 2026

The Canons give one line for each eclipse. For 2026, NASA's published path tables give the path
itself, so the curves can be compared directly:

| Eclipse | Central line and limits |
| ------- | ----------------------- |
| 2026 August 12, total | Within 4 km |
| 2026 February 17, annular, Sun 12° up | Within 6 km |

The February path is a harder test, as a Sun so low magnifies any shift of the shadow about
fivefold on the ground.

The elements themselves are compared with NASA's polynomial Besselian elements for the 2026
eclipses over four hours, and agree within 0.0002 Earth radii with the long-range series.
