//! The sky the renderer draws over the globe: the Sun's light and disc, the Moon's disc and
//! the stars, all as Earth-fixed directions, and the free-globe pose of an eye standing on
//! the ground to look at them.

use maplibre::{
    celestial::{AngularRadius, Disc, EcefDirection, Star, TwilightBand},
    render::view_state::GlobePose,
};

use super::{MapError, MapView};

/// The renderer's vertical field of view, degrees, for the globe seen from afar.
pub const GLOBE_FIELD_OF_VIEW: f64 = 36.869_897_645_844_02;

/// What the sky holds at one instant, in ECEF unit directions from the eye.
#[derive(Clone, Debug, Default)]
pub struct SkyFrame {
    /// Towards the Sun, with its angular radius in degrees.
    pub sun: Option<([f64; 3], f64)>,
    /// Towards the Moon, with its angular radius in degrees.
    pub moon: Option<([f64; 3], f64)>,
    /// The stars and their visual magnitudes, when they changed.
    pub stars: Option<Vec<([f64; 3], f32)>>,
}

impl MapView {
    /// Lights the globe and fills the sky with `sky`, or with `None` returns to the plain
    /// globe with no sky.
    pub fn set_sky(&mut self, sky: Option<SkyFrame>) {
        let direction = |[x, y, z]: [f64; 3]| EcefDirection::new(x, y, z).ok();
        let sky = sky.unwrap_or_else(|| SkyFrame {
            stars: Some(Vec::new()),
            ..SkyFrame::default()
        });
        let sun = sky.sun.and_then(|(d, _)| direction(d));
        self.map.set_sun_direction(sun);
        self.map.set_sun_angular_radius(
            sky.sun
                .filter(|_| sun.is_some())
                .and_then(|(_, radius)| AngularRadius::new(radius).ok()),
        );
        self.map.set_atmosphere(sun.is_some());
        if let Ok(band) = TwilightBand::new(-6.0, 2.0, 0.35) {
            self.map.set_twilight_band(band);
        }
        self.map.set_moon_disc(
            sky.moon
                .and_then(|(d, radius)| Disc::new(direction(d)?, radius).ok()),
        );
        if let Some(stars) = sky.stars {
            self.map.set_stars(
                stars
                    .into_iter()
                    .filter_map(|(d, magnitude)| Star::new(direction(d)?, magnitude, None).ok())
                    .collect(),
            );
        }
    }

    /// The free-globe camera's pose.
    pub fn pose(&self) -> Option<GlobePose> {
        self.map.view_state().globe_pose()
    }

    /// Places the camera at `pose`.
    pub fn set_pose(&mut self, pose: GlobePose) -> Result<(), MapError> {
        self.map
            .restore_globe_pose(pose)
            .map_err(MapError::Navigation)
    }

    /// Sets the vertical field of view, degrees.
    pub fn set_field_of_view(&mut self, degrees: f64) {
        self.map
            .set_vertical_field_of_view(cgmath::Deg(degrees.clamp(0.2, 120.0)));
    }
}
