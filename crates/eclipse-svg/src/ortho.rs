//! The globe seen from far away above one point, as the Canon draws its maps, and the SVG
//! path data of curves and regions on it. The far side is hidden: a curve breaks where it
//! passes behind the limb, and a region is clipped to the visible disc along the limb.

use eclipse_core::geodesy::LatLon;
use std::fmt::Write;

/// Points of each arc drawn for a clipped hemisphere's edges.
const ARC_POINTS: usize = 90;

type Vector = [f64; 3];

/// An orthographic view of the globe centred on one point, drawn as a disc of `radius`
/// around `(cx, cy)` with north up.
#[derive(Clone, Copy, Debug)]
pub struct Ortho {
    centre: Vector,
    east: Vector,
    north: Vector,
    cx: f64,
    cy: f64,
    radius: f64,
}

impl Ortho {
    /// The view from above `centre`, drawn as a disc of `radius` around `(cx, cy)`.
    pub fn new(centre: LatLon, cx: f64, cy: f64, radius: f64) -> Self {
        let lon = centre.lon.to_radians();
        let east = [-lon.sin(), lon.cos(), 0.0];
        let centre = unit(centre);
        Self {
            centre,
            east,
            north: cross(centre, east),
            cx,
            cy,
            radius,
        }
    }

    /// Where `at` shows, or `None` on the far side.
    pub fn project(&self, at: LatLon) -> Option<[f64; 2]> {
        let p = unit(at);
        (dot(p, self.centre) >= 0.0).then(|| self.screen(p))
    }

    fn screen(&self, p: Vector) -> [f64; 2] {
        [
            self.cx + self.radius * dot(p, self.east),
            self.cy - self.radius * dot(p, self.north),
        ]
    }

    /// Path data for the visible stretches of `curves`, leaving out points nearer than
    /// `min_step` to the last one kept, though never a stretch's last point.
    pub fn polyline_d(&self, curves: &[Vec<LatLon>], min_step: f64) -> String {
        let mut d = String::new();
        for curve in curves {
            let mut last: Option<[f64; 2]> = None;
            let mut skipped: Option<[f64; 2]> = None;
            for point in curve {
                match (self.project(*point), last) {
                    (None, _) => {
                        if let Some(q) = skipped.take() {
                            line_to(&mut d, q);
                        }
                        last = None;
                    }
                    (Some(p), None) => {
                        move_to(&mut d, p);
                        last = Some(p);
                    }
                    (Some(p), Some(l)) if distance(l, p) >= min_step => {
                        line_to(&mut d, p);
                        last = Some(p);
                        skipped = None;
                    }
                    (Some(p), Some(_)) => skipped = Some(p),
                }
            }
            if let Some(q) = skipped {
                line_to(&mut d, q);
            }
        }
        d
    }

    /// Path data for the closed `outline`, if it is wholly in view.
    pub fn polygon_d(&self, outline: &[LatLon]) -> Option<String> {
        let points: Option<Vec<[f64; 2]>> = outline.iter().map(|p| self.project(*p)).collect();
        let points = points.filter(|points| points.len() > 2)?;
        Some(closed(&points))
    }

    /// Path data for the part of the visible disc within 90° of `pole`: the night side
    /// when `pole` is the point opposite the Sun.
    pub fn hemisphere_d(&self, pole: LatLon) -> Option<String> {
        let (v, p) = (self.centre, unit(pole));
        let c = dot(v, p);
        if c > 1.0 - 1e-9 {
            return Some(closed(&self.half_circles(self.east, self.north, 2)));
        }
        if c < -1.0 + 1e-9 {
            return None;
        }
        // n lies on both the limb and the hemisphere's edge; w is the edge's point nearest
        // the view centre and u the limb's point inside the hemisphere.
        let n = normalize(cross(v, p));
        let w = normalize(sub(v, scale(p, c)));
        let mut u = normalize(cross(v, n));
        if dot(u, p) < 0.0 {
            u = scale(u, -1.0);
        }
        let mut points = self.half_circles(n, w, 1);
        points.extend(self.half_circles(scale(n, -1.0), u, 1));
        Some(closed(&points))
    }

    /// `halves` half great circles from `from` through `through`, which is 90° from it.
    fn half_circles(&self, from: Vector, through: Vector, halves: usize) -> Vec<[f64; 2]> {
        let count = ARC_POINTS * halves;
        (0..=count)
            .map(|i| {
                let angle = std::f64::consts::PI * (i as f64 / ARC_POINTS as f64);
                let (s, c) = angle.sin_cos();
                self.screen(add(scale(from, c), scale(through, s)))
            })
            .collect()
    }
}

/// The unit vector of `at` on the sphere.
fn unit(at: LatLon) -> Vector {
    let (lat, lon) = (at.lat.to_radians(), at.lon.to_radians());
    [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
}

fn dot(a: Vector, b: Vector) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vector, b: Vector) -> Vector {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add(a: Vector, b: Vector) -> Vector {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Vector, b: Vector) -> Vector {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Vector, k: f64) -> Vector {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn normalize(a: Vector) -> Vector {
    let length = dot(a, a).sqrt();
    if length > 0.0 {
        scale(a, 1.0 / length)
    } else {
        a
    }
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn move_to(d: &mut String, p: [f64; 2]) {
    write!(d, "M{:.1} {:.1}", p[0], p[1]).ok();
}

fn line_to(d: &mut String, p: [f64; 2]) {
    write!(d, "L{:.1} {:.1}", p[0], p[1]).ok();
}

fn closed(points: &[[f64; 2]]) -> String {
    let mut d = String::new();
    for (i, p) in points.iter().enumerate() {
        let command = if i == 0 { 'M' } else { 'L' };
        write!(d, "{command}{:.1} {:.1}", p[0], p[1]).ok();
    }
    d.push('Z');
    d
}

#[cfg(test)]
mod tests;
