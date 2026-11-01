# The shadow

The Moon's shadow is two cones with the Sun and Moon as their ends. Everything on a solar map
comes from where those cones meet the Earth, so the work is to describe them simply and then
intersect them with the ground.

## Sun and Moon

The positions come from [Navigate](https://github.com/luofang34/Navigate)'s analytic series,
with light time and aberration applied, in Earth-fixed axes. The page uses Navigate's long-range
series, fitted to JPL DE431, which spans the years -2000 to 3000.

An ephemeris call is expensive compared with the arithmetic around it. So `eclipse-core` samples
the Sun and Moon every five minutes around an eclipse and interpolates between the samples with
four-point Lagrange polynomials, which agree with Navigate to a few metres.

## The fundamental plane

Besselian elements describe the shadow on a plane, the *fundamental plane*, through the Earth's
centre at right angles to the shadow's axis. Following the *Explanatory Supplement to the
Astronomical Almanac*, its axes are ξ to the east, η to the north and ζ toward the Sun, in
Earth equatorial radii.

The axis crosses the plane at (x, y). The cones are described by their radii on the plane, l1
for the penumbra and l2 for the umbra, and by the tangents of their half angles, f1 and f2.

A point at height ζ above the plane, at distance Δ from the axis, is then:

- inside the penumbra when Δ < L1, where L1 = l1 - ζ tan f1;
- inside the umbra when Δ < |L2|, where L2 = l2 - ζ tan f2.

L2 changes sign at the umbra's point. Short of it L2 is negative, the observer is in the umbra
itself, and the eclipse is total. Past it L2 is positive, the observer is in the cone beyond
the point, and the eclipse is annular.

Besselian elements are often published as polynomials in time. Here they are computed directly
from the interpolated Sun and Moon at each instant asked for.

## Three radii

An eclipse needs a size for the Sun and the Moon, and the Canon's choices are followed here:

| Body | Radius | Used for |
| ---- | ------ | -------- |
| Sun | 696,000 km | Both cones |
| Moon | 0.272488 Earth radii | The penumbra and annular phases: the mean over the limb |
| Moon | 0.272281 Earth radii | Total phases: the mean minimum, at the limb's valleys |

The Moon has two radii because its limb is rough. Sunlight reaches the observer through the
valleys until the last moment, so totality begins later and ends earlier than the mean limb
would give.

## The ground

The Earth is the WGS84 ellipsoid, with flattening 1/298.257223563. A point on the ground has a
position in the fundamental plane at each instant, as the Earth turns under the shadow, and its
circumstances follow by comparing Δ with L1 and L2. Finding the contacts at a place means
searching time for the moments Δ equals those radii.
