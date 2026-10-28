//! What an eclipse does to the sky at the eye, for the renderer: the share of the Sun's disc a
//! solar eclipse leaves uncovered there, and the Earth's shadow a lunar eclipse puts the Moon
//! in, with the Canon's radii.

use eclipse_core::geodesy::{LatLon, surface};

use crate::{
    map::{ShadowFrame, SkyFrame},
    selection::Selected,
};

const ARCMINUTE: f64 = std::f64::consts::PI / 10_800.0;

/// `frame` with the eclipse's light at `site` and `now_tt` added.
pub(crate) fn with_eclipse(
    mut frame: SkyFrame,
    site: LatLon,
    selected: &Selected,
    now_tt: f64,
) -> SkyFrame {
    match selected {
        Selected::Solar { eclipse, .. } => {
            frame.sun_visible = eclipse.elements(now_tt).ok().and_then(|elements| {
                let shadow = elements.shadow(surface(site));
                (shadow.zeta > 0.0).then(|| (1.0 - shadow.obscuration()) as f32)
            });
        }
        Selected::Lunar { eclipse, .. } => {
            let (Some((moon, _)), Ok(at), Ok([penumbra, umbra, _])) = (
                frame.moon,
                eclipse.moon_in_shadow(now_tt),
                eclipse.radii(now_tt),
            ) else {
                return frame;
            };
            // East and north on the sky at the Moon; the shadow's centre lies opposite the
            // Moon's offset from it, at the Moon's distance, so the Moon's parallax holds for
            // both.
            let east = normalize(cross([0.0, 0.0, 1.0], moon));
            let north = cross(moon, east);
            let axis = normalize(
                [0, 1, 2].map(|i| moon[i] - (at.east * east[i] + at.north * north[i]) * ARCMINUTE),
            );
            frame.earth_shadow = Some(ShadowFrame {
                axis,
                umbra_degrees: umbra / 60.0,
                penumbra_degrees: penumbra / 60.0,
            });
        }
    }
    frame
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: [f64; 3]) -> [f64; 3] {
    let length = a.iter().map(|c| c * c).sum::<f64>().sqrt().max(1e-12);
    a.map(|c| c / length)
}
