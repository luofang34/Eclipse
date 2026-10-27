//! The lunar eclipse figure, as the NASA Five Millennium Canon of Lunar Eclipses draws it:
//! the Earth's penumbra and umbra, the Moon at each contact, its path and the ecliptic,
//! north up and east to the left as the sky is seen.

use eclipse_core::{EclipseError, LunarEclipse, MoonInShadow};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke};

use crate::palette;

/// Draws the figure of `eclipse` in `rect`, with the Moon also where it stands at `now_tt`.
pub(crate) fn draw(
    painter: &egui::Painter,
    rect: Rect,
    eclipse: &LunarEclipse,
    now_tt: f64,
) -> Result<(), EclipseError> {
    let [penumbra, umbra, moon] = eclipse.radii(eclipse.greatest_tt)?;
    let places = places(eclipse)?;
    // Arcminutes to points, so the penumbra and every Moon fit.
    let reach = places
        .iter()
        .map(|(_, at)| at.east.hypot(at.north) + moon)
        .fold(penumbra, f64::max);
    let scale = (0.46 * rect.width().min(rect.height())) as f64 / reach;
    let centre = rect.center();
    let to_screen = |east: f64, north: f64| {
        // East is to the left on the sky.
        Pos2::new(
            centre.x - (east * scale) as f32,
            centre.y - (north * scale) as f32,
        )
    };
    painter.rect_filled(rect, 4.0, Color32::WHITE);
    painter.circle_filled(centre, (penumbra * scale) as f32, palette::FIGURE_PENUMBRA);
    painter.circle_filled(centre, (umbra * scale) as f32, palette::FIGURE_UMBRA);
    let thin = Stroke::new(1.0, Color32::BLACK);
    let [east, north] = eclipse.shadow_motion()?;
    let ecliptic = |k: f64| to_screen(east * k * reach, north * k * reach);
    painter.extend(Shape::dashed_line(
        &[ecliptic(-1.0), ecliptic(1.0)],
        thin,
        5.0,
        4.0,
    ));
    for d in [egui::vec2(6.0, 0.0), egui::vec2(0.0, 6.0)] {
        painter.line_segment([centre - d, centre + d], thin);
    }
    let path: Vec<Pos2> = places
        .iter()
        .map(|(_, at)| to_screen(at.east, at.north))
        .collect();
    if let (Some(first), Some(last)) = (path.first(), path.last()) {
        let along = (*last - *first).normalized() * (moon * scale) as f32;
        painter.line_segment([*first - along, *last + along], thin);
    }
    let radius = (moon * scale) as f32;
    let now = eclipse.moon_in_shadow(now_tt)?;
    painter.circle(
        to_screen(now.east, now.north),
        radius,
        Color32::from_rgba_unmultiplied(0xf0, 0xe8, 0xc8, 0x90),
        Stroke::new(1.5, Color32::from_rgb(0x80, 0x60, 0x10)),
    );
    for ((name, _), at) in places.iter().zip(&path) {
        painter.circle_stroke(*at, radius, thin);
        if !name.is_empty() {
            painter.text(
                *at + egui::vec2(0.0, radius + 8.0),
                Align2::CENTER_CENTER,
                *name,
                FontId::proportional(11.0),
                Color32::BLACK,
            );
        }
    }
    painter.text(
        rect.left_top() + egui::vec2(8.0, 6.0),
        Align2::LEFT_TOP,
        "N up, E left",
        FontId::proportional(11.0),
        Color32::DARK_GRAY,
    );
    Ok(())
}

/// The Moon against the shadow at each contact and at greatest eclipse, which is unnamed.
fn places(eclipse: &LunarEclipse) -> Result<Vec<(&'static str, MoonInShadow)>, EclipseError> {
    let c = eclipse.contacts;
    [
        ("P1", c.p1),
        ("U1", c.u1),
        ("U2", c.u2),
        ("", Some(eclipse.greatest_tt)),
        ("U3", c.u3),
        ("U4", c.u4),
        ("P4", c.p4),
    ]
    .into_iter()
    .filter_map(|(name, tt)| tt.map(|tt| (name, tt)))
    .map(|(name, tt)| Ok((name, eclipse.moon_in_shadow(tt)?)))
    .collect()
}
