//! Where the eye stands and where it looks: a free-globe camera pose standing on the
//! renderer's sphere and looking at a direction in the sky, and the flight down to it from
//! the pose over the globe. The pose is the renderer's: its frame has y towards the north
//! pole and z towards longitude zero, and its orientation takes that frame into the
//! camera's, which looks along its negative z with y up.

use cgmath::{InnerSpace, Matrix, Matrix3, Quaternion, Rotation, Vector3};
use eclipse_core::geodesy::LatLon;

/// The eye's height above the ground, metres.
pub(crate) const EYE_HEIGHT_M: f64 = 2.0;
/// How far ahead of a standing eye its pose's target lies, metres.
const STANDING_TARGET_M: f64 = 1000.0;
/// Below this the view would look into the ground, where the pose's target cannot stand.
const LOWEST_ELEVATION: f64 = 0.0;
/// Short of the zenith, where the frame's up is undefined.
const HIGHEST_ELEVATION: f64 = 89.5;

/// A free-globe camera pose, as the renderer stores it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Pose {
    /// Direction from the centre to the point looked at.
    pub(crate) target: [f64; 3],
    /// Height of that point above the sphere, metres.
    pub(crate) target_elevation_meters: f64,
    /// Rotation from the body's frame into the camera's, `[w, x, y, z]`.
    pub(crate) orientation: [f64; 4],
    /// Distance from the eye to the target, metres.
    pub(crate) distance_meters: f64,
}

/// An ECEF vector in the renderer's axes.
pub(crate) fn globe_world([x, y, z]: [f64; 3]) -> Vector3<f64> {
    Vector3::new(y, z, x)
}

/// Up, east and north at `site` on the sphere, in the renderer's axes.
fn local_axes(site: LatLon) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
    let (lat, lon) = (site.lat.to_radians(), site.lon.to_radians());
    let up = [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()];
    let east = [-lon.sin(), lon.cos(), 0.0];
    let (up, east) = (globe_world(up), globe_world(east));
    (up, east, up.cross(east))
}

/// Azimuth east of north and elevation, degrees, of the ECEF `direction` against the
/// sphere's horizon at `site`, the horizon the renderer draws.
pub(crate) fn on_sphere(site: LatLon, direction: [f64; 3]) -> [f64; 2] {
    let (up, east, north) = local_axes(site);
    let d = globe_world(direction).normalize();
    let azimuth = d
        .dot(east)
        .atan2(d.dot(north))
        .to_degrees()
        .rem_euclid(360.0);
    [azimuth, d.dot(up).clamp(-1.0, 1.0).asin().to_degrees()]
}

/// Azimuth east of north and elevation, degrees, of the ECEF `direction` against the true
/// horizon at `site`, square to the ellipsoid.
pub(crate) fn horizontal(site: LatLon, direction: [f64; 3]) -> [f64; 2] {
    let up = eclipse_core::geodesy::up(site);
    let lon = site.lon.to_radians();
    let east = [-lon.sin(), lon.cos(), 0.0];
    let north = [
        up[1] * east[2] - up[2] * east[1],
        up[2] * east[0] - up[0] * east[2],
        up[0] * east[1] - up[1] * east[0],
    ];
    let along = |axis: [f64; 3]| axis.iter().zip(direction).map(|(a, d)| a * d).sum::<f64>();
    let azimuth = along(east)
        .atan2(along(north))
        .to_degrees()
        .rem_euclid(360.0);
    [azimuth, along(up).clamp(-1.0, 1.0).asin().to_degrees()]
}

/// The elevation a standing view can take, degrees.
pub(crate) fn standing_elevation(elevation: f64) -> f64 {
    elevation.clamp(LOWEST_ELEVATION, HIGHEST_ELEVATION)
}

/// The pose standing `EYE_HEIGHT_M` above `site` on a sphere of `radius` metres, looking at
/// `azimuth` and `elevation` degrees on the sphere's horizon, level with it.
pub(crate) fn standing(site: LatLon, azimuth: f64, elevation: f64, radius: f64) -> Pose {
    let (up, east, north) = local_axes(site);
    let (a, e) = (
        azimuth.to_radians(),
        standing_elevation(elevation).to_radians(),
    );
    let forward = (east * a.sin() + north * a.cos()) * e.cos() + up * e.sin();
    pose_at(
        up * (radius + EYE_HEIGHT_M),
        forward,
        up,
        STANDING_TARGET_M,
        radius,
    )
}

/// The pose with its eye at `eye`, in the renderer's metres, looking along `forward` with
/// `up` towards the top of the frame, its target `distance` metres ahead.
fn pose_at(
    eye: Vector3<f64>,
    forward: Vector3<f64>,
    up: Vector3<f64>,
    distance: f64,
    radius: f64,
) -> Pose {
    let forward = forward.normalize();
    let side = forward.cross(up);
    let right = if side.magnitude2() > 1e-12 {
        side.normalize()
    } else {
        forward.cross(Vector3::unit_y()).normalize()
    };
    let camera_up = right.cross(forward);
    // The camera looks along its negative z with y up; the rows take the body's frame into it.
    let orientation = Quaternion::from(Matrix3::from_cols(right, camera_up, -forward).transpose());
    aimed(eye, orientation, distance, radius)
}

/// The pose of an eye at `eye` turned by `orientation`, its target `distance` ahead.
fn aimed(eye: Vector3<f64>, orientation: Quaternion<f64>, distance: f64, radius: f64) -> Pose {
    let target = eye + forward_of(orientation) * distance;
    Pose {
        target: target.normalize().into(),
        target_elevation_meters: target.magnitude() - radius,
        orientation: [
            orientation.s,
            orientation.v.x,
            orientation.v.y,
            orientation.v.z,
        ],
        distance_meters: distance,
    }
}

fn forward_of(orientation: Quaternion<f64>) -> Vector3<f64> {
    -orientation.conjugate().rotate_vector(Vector3::unit_z())
}

/// Where a pose's eye stands and which way it looks, in the renderer's metres, as
/// Sokoly-App's `pose::eye_world_m` reads a pose.
pub(crate) fn eye_and_forward(pose: &Pose, radius: f64) -> (Vector3<f64>, Vector3<f64>) {
    let [w, x, y, z] = pose.orientation;
    let forward = forward_of(Quaternion::new(w, x, y, z).normalize());
    let target = Vector3::from(pose.target) * (radius + pose.target_elevation_meters);
    (target - forward * pose.distance_meters, forward)
}

/// The pose `t` of the way from `from` to `to`: the eye along the great circle between
/// them, its height falling geometrically, and its orientation turning evenly.
pub(crate) fn between(from: &Pose, to: &Pose, t: f64, radius: f64) -> Pose {
    let rotation = |pose: &Pose| {
        let [w, x, y, z] = pose.orientation;
        Quaternion::new(w, x, y, z).normalize()
    };
    let (eye0, eye1) = (
        eye_and_forward(from, radius).0,
        eye_and_forward(to, radius).0,
    );
    let (q0, q1) = (rotation(from), rotation(to));
    let height = |eye: Vector3<f64>| (eye.magnitude() - radius).max(EYE_HEIGHT_M);
    let (h0, h1) = (height(eye0), height(eye1));
    let direction = slerp_vector(eye0.normalize(), eye1.normalize(), t);
    let h = (h0.ln() * (1.0 - t) + h1.ln() * t).exp();
    // The shorter way round.
    let q1 = if q0.dot(q1) < 0.0 { -q1 } else { q1 };
    // A target nearer than the eye's height stays above the ground whichever way it looks.
    aimed(
        direction * (radius + h),
        q0.slerp(q1, t).normalize(),
        0.5 * h,
        radius,
    )
}

fn slerp_vector(a: Vector3<f64>, b: Vector3<f64>, t: f64) -> Vector3<f64> {
    let angle = a.dot(b).clamp(-1.0, 1.0).acos();
    if angle < 1e-9 {
        return a;
    }
    (a * ((1.0 - t) * angle).sin() + b * (t * angle).sin()) / angle.sin()
}

/// Eases in and out.
pub(crate) fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests;
