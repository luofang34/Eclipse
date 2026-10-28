//! The sky from a place on the ground: the camera flies down from the globe to stand there
//! and look at the Sun or the Moon, which Navigate places as that place sees them, with the
//! Moon seen from the place itself rather than the Earth's centre.

use eclipse_core::{
    EARTH_RADIUS_M,
    geodesy::{LatLon, surface},
    time::tt_epoch,
};
use maplibre::{projection::globe::EARTH_RADIUS_METERS, render::view_state::GlobePose};

use crate::{
    eye::{self, Pose},
    map::SkyFrame,
};

mod drawing;
mod eclipse;

pub(crate) use drawing::draw;
pub(crate) use eclipse::with_eclipse;

/// How long the flight down from the globe takes, seconds, as Sokoly-Demos' sky cases fly in.
const FLIGHT_S: f64 = 6.0;
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
    from: Pose,
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
                from: Pose {
                    target: from.target,
                    target_elevation_meters: from.target_elevation_meters,
                    orientation: from.orientation,
                    distance_meters: from.distance_meters,
                },
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
            // Against the sphere's horizon, the one the camera is posed on.
            let [azimuth, elevation] = eye::on_sphere(self.site, direction);
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
        // The view is kept at or above the horizon, so the target stays above the ground.
        self.elevation = eye::standing_elevation(self.elevation);
        let standing = eye::standing(self.site, self.azimuth, self.elevation, EARTH_RADIUS_METERS);
        let pose = match &mut self.flight {
            None => standing,
            Some(flight) => {
                let started = *flight.started_s.get_or_insert(now_s);
                let progress = ((now_s - started) / FLIGHT_S).clamp(0.0, 1.0);
                if progress >= 1.0 {
                    self.flight = None;
                    standing
                } else {
                    eye::between(
                        &flight.from,
                        &standing,
                        eye::smooth(progress),
                        EARTH_RADIUS_METERS,
                    )
                }
            }
        };
        GlobePose {
            target: pose.target,
            target_elevation_meters: pose.target_elevation_meters,
            orientation: pose.orientation,
            distance_meters: pose.distance_meters,
        }
    }

    /// The field of view for this frame: widening out from the globe's while flying.
    pub(crate) fn frame_field_of_view(&self, now_s: f64) -> f64 {
        let progress = self.flight.as_ref().and_then(|f| {
            f.started_s
                .map(|s| ((now_s - s) / FLIGHT_S).clamp(0.0, 1.0))
        });
        match progress {
            Some(p) => {
                let p = eye::smooth(p);
                crate::map::GLOBE_FIELD_OF_VIEW * (1.0 - p) + self.field_of_view * p
            }
            None if self.flying() => crate::map::GLOBE_FIELD_OF_VIEW,
            None => self.field_of_view,
        }
    }

    /// The Sun, the Moon and, when they moved, the stars at `jd_tt` from the site.
    pub(crate) fn sky(&mut self, jd_tt: f64) -> SkyFrame {
        let epoch = tt_epoch(jd_tt);
        let eye = surface(self.site).map(|c| c * EARTH_RADIUS_M);
        let sight = |s: navigate_celestial::EarthFixedSight| {
            (s.direction, s.angular_radius_rad.to_degrees())
        };
        let stale = self
            .stars_at
            .is_none_or(|at| (jd_tt - at).abs() >= STAR_REFRESH_DAYS);
        let stars = stale.then(|| {
            self.stars_at = Some(jd_tt);
            navigate_celestial::stars_ecef_tt(epoch, FAINTEST_MAGNITUDE)
                .map(|stars| {
                    stars
                        .into_iter()
                        .map(|star| (star.direction, star.v_magnitude as f32))
                        .collect()
                })
                .unwrap_or_default()
        });
        SkyFrame {
            sun: navigate_celestial::sun_ecef_tt(epoch, Some(eye))
                .ok()
                .map(sight),
            moon: navigate_celestial::moon_ecef_tt(epoch, Some(eye))
                .ok()
                .map(sight),
            stars,
            ..SkyFrame::default()
        }
    }
}
