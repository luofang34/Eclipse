# Curves on the ground

A curve on a solar map is not found by testing places. Each one is a family of points that
exists from moment to moment, traced through the eclipse.

At each instant, a family gives the points of its curve that exist then. The central line has
at most one point, where the axis meets the ground. Other families can have several at once,
and they are found by searching around a circle for changes of sign.

Points of consecutive instants are then joined into polylines. Three things make this harder
than it sounds, and the same few constants handle them:

- **Curves run fast at their ends.** Near sunrise and sunset, the shadow leaves the Earth at a
  slant and a curve can cover a great distance in a second. Instants are added until
  consecutive points lie within about 50 km of each other, and the step can shrink to a quarter
  of a second where a curve begins or ends.
- **Families have several branches.** Points further apart than 0.15 Earth radii in consecutive
  instants are not taken for the same curve.
- **Curves have gaps at the seams.** Ends closer than about 20 km are joined into one curve.

The outline of the shadow at a moment, drawn as the time bar moves, is computed the same way
for one instant: a ring around the axis, intersected with the ground.

## From the curve to the screen

The curves are computed once, when an eclipse is opened, as latitudes and longitudes. Each frame
projects them through the camera of the frame under them, so they stay fixed to the ground as
the globe turns.

Dashes need the same care. A dash pattern laid out on the screen would slide along the curve
with every move, because the pattern would restart from the first visible point. Here dashes
are laid out by distance along the whole curve, and they stay put on the ground. The spacing is
a power of two in kilometres, chosen for the zoom, so it keeps close to the same size on the
screen.
