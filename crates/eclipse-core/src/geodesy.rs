//! Points on the WGS84 ellipsoid, in Earth equatorial radii and ECEF axes.

use crate::FLATTENING;

/// The squared ratio of the polar to the equatorial radius.
const POLAR_SQUARED: f64 = (1.0 - FLATTENING) * (1.0 - FLATTENING);

/// A geodetic position in degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LatLon {
    /// Geodetic latitude, positive north.
    pub lat: f64,
    /// Longitude, positive east, in `[-180, 180]`.
    pub lon: f64,
}

/// The geodetic position of `p`, a point on the ellipsoid.
pub fn lat_lon(p: [f64; 3]) -> LatLon {
    LatLon {
        lat: (p[2] / POLAR_SQUARED).atan2(p[0].hypot(p[1])).to_degrees(),
        lon: p[1].atan2(p[0]).to_degrees(),
    }
}

/// The point at sea level below `at`.
pub fn surface(at: LatLon) -> [f64; 3] {
    let (lat, lon) = (at.lat.to_radians(), at.lon.to_radians());
    let e2 = 1.0 - POLAR_SQUARED;
    let n = 1.0 / (1.0 - e2 * lat.sin().powi(2)).sqrt();
    [
        n * lat.cos() * lon.cos(),
        n * lat.cos() * lon.sin(),
        n * POLAR_SQUARED * lat.sin(),
    ]
}

/// The upward unit normal at `at`.
pub fn up(at: LatLon) -> [f64; 3] {
    let (lat, lon) = (at.lat.to_radians(), at.lon.to_radians());
    [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
}

/// The altitude in degrees of the body at `body` seen from the sea-level point `at`,
/// without refraction.
pub fn altitude(at: LatLon, body: [f64; 3]) -> f64 {
    let p = surface(at);
    let d = sub(body, p);
    (dot(d, up(at)) / norm(d))
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees()
}

/// Where the body at `body` stands overhead: the point whose upward normal points at it.
pub fn overhead(body: [f64; 3]) -> LatLon {
    let d = norm(body);
    LatLon {
        lat: (body[2] / d).clamp(-1.0, 1.0).asin().to_degrees(),
        lon: body[1].atan2(body[0]).to_degrees(),
    }
}

/// Great-circle distance in kilometres on the mean sphere.
pub fn distance_km(a: LatLon, b: LatLon) -> f64 {
    let (u, v) = (up(a), up(b));
    let angle = norm(cross(u, v)).atan2(dot(u, v));
    angle * 6_371.0
}

/// The distance in km from `p` to the polyline `curve`, following each segment's chord.
pub fn distance_to_polyline(p: LatLon, curve: &[LatLon]) -> f64 {
    let mut best = f64::INFINITY;
    for pair in curve.windows(2) {
        let (a, b) = (surface(pair[0]), surface(pair[1]));
        for k in 0..=32 {
            let f = f64::from(k) / 32.0;
            let q = [0, 1, 2].map(|i| a[i] + f * (b[i] - a[i]));
            best = best.min(distance_km(p, lat_lon(q)));
        }
    }
    if let [only] = curve {
        best = distance_km(p, *only);
    }
    best
}

pub(crate) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(crate) fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

pub(crate) fn unit(a: [f64; 3]) -> [f64; 3] {
    let n = norm(a);
    a.map(|component| component / n)
}

pub(crate) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(crate) fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    a.map(|component| component * s)
}

pub(crate) fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Where the line `origin + λ direction` meets the ellipsoid, the root nearest `+direction`;
/// `None` when it misses.
pub(crate) fn hit(origin: [f64; 3], direction: [f64; 3]) -> Option<f64> {
    let c = 1.0 / (1.0 - FLATTENING);
    let so = [origin[0], origin[1], origin[2] * c];
    let sd = [direction[0], direction[1], direction[2] * c];
    let (a, b, k) = (dot(sd, sd), 2.0 * dot(sd, so), dot(so, so) - 1.0);
    let disc = b * b - 4.0 * a * k;
    (disc >= 0.0).then(|| (-b + disc.sqrt()) / (2.0 * a))
}
