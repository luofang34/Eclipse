//! The sky from a place on the ground: the camera flies down from the globe to stand there
//! and look at the Sun or the Moon, which Navigate places as that place sees them, with the
//! Moon seen from the place itself rather than the Earth's centre.

use cgmath::{InnerSpace, Matrix, Matrix3, Quaternion, Rotation, Vector3};
use eclipse_core::{EARTH_RADIUS_M, geodesy::LatLon, geodesy::surface, time::civil_epoch};
use maplibre::{
    celestial::ecef_to_globe_world, projection::globe::EARTH_RADIUS_METERS,
    render::view_state::GlobePose,
};

use crate::map::SkyFrame;

mod drawing;

pub(crate) use drawing::draw;

/// The eye's height above the ground, metres.
const EYE_HEIGHT_M: f64 = 2.0;
/// How far ahead of a standing eye its pose's target lies, metres.
const STANDING_TARGET_M: f64 = 1000.0;
/// How long the flight down from the globe takes, seconds.
const FLIGHT_S: f64 = 5.0;
/// The faintest stars shown, about what the eye sees in a dark sky.
const FAINTEST_MAGNITUDE: f64 = 6.0;
/// The stars are placed again once the clock has moved this far, days: the sky turns a
/// quarter of a degree in it.
const STAR_REFRESH_DAYS: f64 = 60.0 / 86_400.0;
/// The view's field of view when it lands, degrees.
const LANDING_FIELD_OF_VIEW: f64 = 40.0;

/// What the eye looks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Body {
    Sun,
    Moon,
}

/// The view from a place on the ground.
pub(crate) struct SkyView {
    /// Where the eye stands.
    pub(crate) site: LatLon,
    /// Degrees east of north, and above the horizon, of the view's centre.
    pub(crate) azimuth: f64,
    pub(crate) elevation: f64,
    /// Vertical field of view, degrees.
    pub(crate) field_of_view: f64,
    /// The body the view keeps centred, until the viewer looks away.
    pub(crate) follow: Option<Body>,
    /// The Sun and Moon last drawn, for the panel's account.
    pub(crate) shown: Option<SkyFrame>,
    flight: Option<Flight>,
    stars_at: Option<f64>,
}

/// The camera's way down: from the pose it had to the standing one.
struct Flight {
    from: GlobePose,
    started_s: Option<f64>,
}

impl SkyView {
    /// A view standing at `site` that follows `body`, flying down from the pose `from`.
    pub(crate) fn new(site: LatLon, body: Body, from: Option<GlobePose>) -> Self {
        Self {
            site,
            azimuth: 0.0,
            elevation: 30.0,
            field_of_view: LANDING_FIELD_OF_VIEW,
            follow: Some(body),
            flight: from.map(|from| Flight {
                from,
                started_s: None,
            }),
            shown: None,
            stars_at: None,
        }
    }

    /// Whether the camera is still on its way down.
    pub(crate) fn flying(&self) -> bool {
        self.flight.is_some()
    }

    /// Turns the view to the body it follows at `jd_tt`.
    pub(crate) fn track(&mut self, sky: &SkyFrame) {
        let toward = match self.follow {
            Some(Body::Sun) => sky.sun,
            Some(Body::Moon) => sky.moon,
            None => None,
        };
        if let Some((direction, _)) = toward {
            let [azimuth, elevation] = horizontal(self.site, direction);
            self.azimuth = azimuth;
            self.elevation = elevation;
        }
    }

    /// Looks `[right, up]` degrees away, which stops following.
    pub(crate) fn look(&mut self, by: [f64; 2]) {
        self.follow = None;
        self.azimuth = (self.azimuth + by[0]).rem_euclid(360.0);
        self.elevation += by[1];
    }

    /// The camera's pose at `now_s` seconds of page time, partway down while flying.
    pub(crate) fn pose(&mut self, now_s: f64) -> GlobePose {
        let standing = self.standing_pose();
        let Some(flight) = &mut self.flight else {
            return standing;
        };
        let started = *flight.started_s.get_or_insert(now_s);
        let progress = ((now_s - started) / FLIGHT_S).clamp(0.0, 1.0);
        if progress >= 1.0 {
            self.flight = None;
            return standing;
        }
        between(&flight.from, &standing, smooth(progress))
    }

    /// The field of view for this frame: widening out from the globe's while flying.
    pub(crate) fn frame_field_of_view(&self, now_s: f64) -> f64 {
        let progress = self.flight.as_ref().and_then(|f| {
            f.started_s
                .map(|s| ((now_s - s) / FLIGHT_S).clamp(0.0, 1.0))
        });
        match progress {
            Some(p) => {
                let p = smooth(p);
                crate::map::GLOBE_FIELD_OF_VIEW * (1.0 - p) + self.field_of_view * p
            }
            None if self.flying() => crate::map::GLOBE_FIELD_OF_VIEW,
            None => self.field_of_view,
        }
    }

    /// The pose standing at the site and looking at the view's azimuth and elevation. The
    /// view is kept at or above the horizon, so the target stays above the ground.
    fn standing_pose(&mut self) -> GlobePose {
        self.elevation = self.elevation.clamp(0.0, 89.5);
        let (up, east, north) = local_axes(self.site);
        let (azimuth, elevation) = (self.azimuth.to_radians(), self.elevation.to_radians());
        let forward =
            (east * azimuth.sin() + north * azimuth.cos()) * elevation.cos() + up * elevation.sin();
        let eye = up * (EARTH_RADIUS_METERS + EYE_HEIGHT_M);
        pose_at(eye, forward, up, STANDING_TARGET_M)
    }

    /// The Sun, the Moon and, when they moved, the stars at `jd_tt` from the site.
    pub(crate) fn sky(&mut self, jd_tt: f64) -> SkyFrame {
        let epoch = civil_epoch(jd_tt);
        let eye = surface(self.site).map(|c| c * EARTH_RADIUS_M);
        let sight = |s: navigate_celestial::EarthFixedSight| {
            (s.direction, s.angular_radius_rad.to_degrees())
        };
        let stale = self
            .stars_at
            .is_none_or(|at| (jd_tt - at).abs() >= STAR_REFRESH_DAYS);
        let stars = stale.then(|| {
            self.stars_at = Some(jd_tt);
            navigate_celestial::stars_ecef(epoch, FAINTEST_MAGNITUDE)
                .map(|stars| {
                    stars
                        .into_iter()
                        .map(|star| (star.direction, star.v_magnitude as f32))
                        .collect()
                })
                .unwrap_or_default()
        });
        SkyFrame {
            sun: navigate_celestial::sun_ecef(epoch, Some(eye))
                .ok()
                .map(sight),
            moon: navigate_celestial::moon_ecef(epoch, Some(eye))
                .ok()
                .map(sight),
            stars,
        }
    }
}

/// Azimuth east of north and elevation, degrees, of the ECEF `direction` seen from `site`.
pub(crate) fn horizontal(site: LatLon, direction: [f64; 3]) -> [f64; 2] {
    let up = eclipse_core::geodesy::up(site);
    let lon = site.lon.to_radians();
    let east = [-lon.sin(), lon.cos(), 0.0];
    let north = cross(up, east);
    let along = |axis: [f64; 3]| axis.iter().zip(direction).map(|(a, d)| a * d).sum::<f64>();
    let azimuth = along(east)
        .atan2(along(north))
        .to_degrees()
        .rem_euclid(360.0);
    [azimuth, along(up).clamp(-1.0, 1.0).asin().to_degrees()]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Up, east and north at `site` on the renderer's sphere, in globe world axes.
fn local_axes(site: LatLon) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
    let (lat, lon) = (site.lat.to_radians(), site.lon.to_radians());
    let up = Vector3::new(lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin());
    let east = Vector3::new(-lon.sin(), lon.cos(), 0.0);
    let north = up.cross(east);
    (
        ecef_to_globe_world(up),
        ecef_to_globe_world(east),
        ecef_to_globe_world(north),
    )
}

/// The pose with its eye at `eye`, in globe world metres, looking along `forward` with
/// `up` towards the top of the frame, its target `distance` metres ahead.
fn pose_at(eye: Vector3<f64>, forward: Vector3<f64>, up: Vector3<f64>, distance: f64) -> GlobePose {
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
    let target = eye + forward * distance;
    GlobePose {
        target: target.normalize().into(),
        target_elevation_meters: target.magnitude() - EARTH_RADIUS_METERS,
        orientation: [
            orientation.s,
            orientation.v.x,
            orientation.v.y,
            orientation.v.z,
        ],
        distance_meters: distance,
    }
}

/// Where a pose's eye stands and where it looks, in globe world metres.
fn eye_and_forward(pose: &GlobePose) -> (Vector3<f64>, Vector3<f64>, Quaternion<f64>) {
    let [w, x, y, z] = pose.orientation;
    let orientation = Quaternion::new(w, x, y, z).normalize();
    let forward = -orientation.conjugate().rotate_vector(Vector3::unit_z());
    let target = Vector3::from(pose.target) * (EARTH_RADIUS_METERS + pose.target_elevation_meters);
    (
        target - forward * pose.distance_meters,
        forward,
        orientation,
    )
}

/// The pose `t` of the way from `from` to `to`: the eye along the great circle between
/// them, its height falling geometrically, and its orientation turning evenly.
fn between(from: &GlobePose, to: &GlobePose, t: f64) -> GlobePose {
    let (eye0, _, q0) = eye_and_forward(from);
    let (eye1, _, q1) = eye_and_forward(to);
    let (h0, h1) = (
        (eye0.magnitude() - EARTH_RADIUS_METERS).max(EYE_HEIGHT_M),
        (eye1.magnitude() - EARTH_RADIUS_METERS).max(EYE_HEIGHT_M),
    );
    let direction = slerp_vector(eye0.normalize(), eye1.normalize(), t);
    let height = (h0.ln() * (1.0 - t) + h1.ln() * t).exp();
    let eye = direction * (EARTH_RADIUS_METERS + height);
    // The shorter way round.
    let q1 = if q0.dot(q1) < 0.0 { -q1 } else { q1 };
    let orientation = q0.slerp(q1, t).normalize();
    let forward = -orientation.conjugate().rotate_vector(Vector3::unit_z());
    // A target nearer than the eye's height stays above the ground whichever way it looks.
    let distance = 0.5 * height;
    let target = eye + forward * distance;
    GlobePose {
        target: target.normalize().into(),
        target_elevation_meters: target.magnitude() - EARTH_RADIUS_METERS,
        orientation: [
            orientation.s,
            orientation.v.x,
            orientation.v.y,
            orientation.v.z,
        ],
        distance_meters: distance,
    }
}

fn slerp_vector(a: Vector3<f64>, b: Vector3<f64>, t: f64) -> Vector3<f64> {
    let angle = a.dot(b).clamp(-1.0, 1.0).acos();
    if angle < 1e-9 {
        return a;
    }
    (a * ((1.0 - t) * angle).sin() + b * (t * angle).sin()) / angle.sin()
}

/// Eases in and out.
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}
