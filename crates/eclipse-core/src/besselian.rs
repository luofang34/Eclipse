//! Besselian geometry of a solar eclipse: the fundamental plane through the Earth's centre
//! at right angles to the shadow axis, and the shadow cones that cross it.
//!
//! Following the *Explanatory Supplement to the Astronomical Almanac*, the plane's axes
//! are ξ to the east, η to the north and ζ toward the Sun, in Earth equatorial radii. A
//! point at height ζ lies inside the penumbra when its distance Δ from the axis is below
//! L1 = l1 - ζ tan f1, and inside the umbra when below |L2|, L2 = l2 - ζ tan f2, which is
//! negative where the eclipse is total.

use crate::{
    Bodies,
    geodesy::{add, cross, dot, hit, norm, scale, sub, unit},
};

/// The Sun's radius in Earth equatorial radii: 959.63 arcseconds at 1 au, as the NASA
/// Five Millennium Canon uses.
pub const SUN_RADIUS: f64 = 696_000.0 / 6_378.137;
/// The Moon's radius for the penumbra and annular phases: the Canon's mean over the limb.
pub const K_PENUMBRA: f64 = 0.272_488;
/// The Moon's radius for total phases: the Canon's mean minimum, at the limb's valleys.
pub const K_UMBRA: f64 = 0.272_281;

/// The fundamental plane and shadow cones at one instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Elements {
    /// Unit ξ axis, ECEF.
    pub east: [f64; 3],
    /// Unit η axis, ECEF.
    pub north: [f64; 3],
    /// Unit ζ axis, the shadow axis toward the Sun, ECEF.
    pub axis: [f64; 3],
    /// The axis crosses the plane at (x, y).
    pub x: f64,
    /// See [`Elements::x`].
    pub y: f64,
    /// Penumbral radius on the plane.
    pub l1: f64,
    /// Umbral radius on the plane; negative for a total eclipse.
    pub l2: f64,
    /// Tangent of the penumbral cone's half angle.
    pub tan_f1: f64,
    /// Tangent of the umbral cone's half angle.
    pub tan_f2: f64,
    /// The Sun, ECEF, for altitudes.
    pub sun: [f64; 3],
}

/// A cone of the shadow: the points at distance |l - ζ tan f| from the axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cone {
    l: f64,
    tan_f: f64,
}

impl Elements {
    /// The elements for the Sun and Moon of `bodies`.
    pub fn new(bodies: &Bodies) -> Self {
        let to_sun = sub(bodies.sun, bodies.moon);
        let separation = norm(to_sun);
        let axis = scale(to_sun, 1.0 / separation);
        let north = unit(sub([0.0, 0.0, 1.0], scale(axis, axis[2])));
        let east = cross(north, axis);
        let (x, y, z) = (
            dot(bodies.moon, east),
            dot(bodies.moon, north),
            dot(bodies.moon, axis),
        );
        let sin_f1 = (SUN_RADIUS + K_PENUMBRA) / separation;
        let sin_f2 = (SUN_RADIUS - K_UMBRA) / separation;
        let tan_f1 = sin_f1 / (1.0 - sin_f1 * sin_f1).sqrt();
        let tan_f2 = sin_f2 / (1.0 - sin_f2 * sin_f2).sqrt();
        // The vertices lie K / sin f from the Moon's centre along the axis.
        let l1 = (z + K_PENUMBRA / sin_f1) * tan_f1;
        let l2 = (z - K_UMBRA / sin_f2) * tan_f2;
        Self {
            east,
            north,
            axis,
            x,
            y,
            l1,
            l2,
            tan_f1,
            tan_f2,
            sun: bodies.sun,
        }
    }

    /// `p` in the plane's axes.
    pub fn project(&self, p: [f64; 3]) -> [f64; 3] {
        [dot(p, self.east), dot(p, self.north), dot(p, self.axis)]
    }

    /// The point of the Earth's sunward surface above (ξ, η), if the Earth is there.
    pub fn surface(&self, xi: f64, eta: f64) -> Option<[f64; 3]> {
        let origin = add(scale(self.east, xi), scale(self.north, eta));
        hit(origin, self.axis).map(|zeta| add(origin, scale(self.axis, zeta)))
    }

    /// The point of the Earth's outline, where the Sun is on the horizon, at angle `theta`
    /// on the plane from the ξ axis.
    pub fn outline(&self, theta: f64) -> [f64; 3] {
        let c2 = (1.0 - crate::FLATTENING).powi(2);
        // The ellipsoid seen along the axis is an ellipse, its η semi-axis shortened.
        let b = (1.0 - (1.0 - c2) * self.north[2] * self.north[2]).sqrt();
        let (xi, eta) = (theta.cos(), b * theta.sin());
        let origin = add(scale(self.east, xi), scale(self.north, eta));
        // At tangency the line touches the ellipsoid once, at its discriminant's zero.
        let k = 1.0 / c2;
        let a = 1.0 + (k - 1.0) * self.axis[2] * self.axis[2];
        let half_b = dot(origin, self.axis) + (k - 1.0) * origin[2] * self.axis[2];
        add(origin, scale(self.axis, -half_b / a))
    }

    /// The penumbral cone.
    pub fn penumbra(&self) -> Cone {
        Cone {
            l: self.l1,
            tan_f: self.tan_f1,
        }
    }

    /// The umbral cone, signed so its radius is positive where the eclipse is total.
    pub fn umbra(&self) -> Cone {
        Cone {
            l: -self.l2,
            tan_f: -self.tan_f2,
        }
    }

    /// The cone of points that see the Sun's diameter covered by `magnitude`.
    pub fn magnitude_cone(&self, magnitude: f64) -> Cone {
        Cone {
            l: self.l1 - magnitude * (self.l1 + self.l2),
            tan_f: self.tan_f1 - magnitude * (self.tan_f1 + self.tan_f2),
        }
    }

    /// How the Sun and Moon stand from the point `p`.
    pub fn shadow(&self, p: [f64; 3]) -> Shadow {
        let [xi, eta, zeta] = self.project(p);
        Shadow {
            delta: (self.x - xi).hypot(self.y - eta),
            l1: self.l1 - zeta * self.tan_f1,
            l2: self.l2 - zeta * self.tan_f2,
            zeta,
        }
    }
}

impl Cone {
    /// The signed radius at height `zeta`.
    pub fn radius(&self, zeta: f64) -> f64 {
        self.l - zeta * self.tan_f
    }
}

/// The shadow as seen from one point at one instant, in plane units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    /// Distance from the axis.
    pub delta: f64,
    /// Penumbral radius at the point.
    pub l1: f64,
    /// Umbral radius at the point; negative inside a total eclipse's cone.
    pub l2: f64,
    /// The point's height toward the Sun; negative on the night side.
    pub zeta: f64,
}

impl Shadow {
    /// The fraction of the Sun's diameter the Moon covers; negative outside the penumbra.
    pub fn magnitude(&self) -> f64 {
        (self.l1 - self.delta) / (self.l1 + self.l2)
    }

    /// The Moon's apparent diameter over the Sun's: the magnitude of a total or annular
    /// eclipse, as the Canon gives it for central eclipses.
    pub fn diameter_ratio(&self) -> f64 {
        (self.l1 - self.l2) / (self.l1 + self.l2)
    }

    /// The fraction of the Sun's disc the Moon covers.
    pub fn obscuration(&self) -> f64 {
        let sun = 0.5 * (self.l1 + self.l2);
        let moon = 0.5 * (self.l1 - self.l2);
        let d = self.delta;
        if d >= sun + moon {
            return 0.0;
        }
        if d <= (moon - sun).abs() {
            return (moon / sun).powi(2).min(1.0);
        }
        // The lens where two discs overlap.
        let a = ((d * d + sun * sun - moon * moon) / (2.0 * d * sun))
            .clamp(-1.0, 1.0)
            .acos();
        let b = ((d * d + moon * moon - sun * sun) / (2.0 * d * moon))
            .clamp(-1.0, 1.0)
            .acos();
        let lens = sun * sun * (a - a.sin() * a.cos()) + moon * moon * (b - b.sin() * b.cos());
        lens / (std::f64::consts::PI * sun * sun)
    }

    /// Whether the point is inside the umbra or antumbra.
    pub fn central(&self) -> bool {
        self.delta < self.l2.abs()
    }
}

#[cfg(test)]
mod tests;
