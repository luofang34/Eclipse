//! A solar eclipse's map as the Canon draws it, on the globe seen from above greatest
//! eclipse, with the night side, the penumbra's edge and the umbra moving across it.

use eclipse_core::{
    EclipseError, SolarEclipse, SolarMap,
    geodesy::{LatLon, overhead},
    time::{CalendarTime, ut1},
};

use crate::{
    document::{Animation, EMPTY_D, Svg},
    ortho::Ortho,
};

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 900.0;
const RADIUS: f64 = 370.0;
const CENTRE: [f64; 2] = [400.0, 470.0];
/// Frames over the eclipse, and how long the loop plays.
const ANIMATION: Animation = Animation {
    frames: 120,
    seconds: 24.0,
};
/// Coastline points nearer than this on the page add nothing a reader sees.
const BASEMAP_STEP: f64 = 0.8;

const PENUMBRA: &str = "#128a2a";
const MAX_RISE_SET: &str = "#e01e84";
const UMBRA: &str = "#1d3fd8";

/// Coastlines and borders to draw under the eclipse.
#[derive(Clone, Debug, Default)]
pub struct Basemap {
    /// Land outlines.
    pub land: Vec<Vec<LatLon>>,
    /// Country borders.
    pub borders: Vec<Vec<LatLon>>,
}

/// The animated SVG map of `eclipse`, whose curves are `map`, over `basemap`.
pub fn solar_svg(
    eclipse: &SolarEclipse,
    map: &SolarMap,
    basemap: &Basemap,
) -> Result<String, EclipseError> {
    let view = Ortho::new(eclipse.greatest, CENTRE[0], CENTRE[1], RADIUS);
    let mut svg = Svg::new(WIDTH, HEIGHT);
    globe(&mut svg, &view, basemap);
    moving(&mut svg, &view, eclipse)?;
    curves(&mut svg, &view, eclipse, map);
    if let Some(p) = view.project(eclipse.greatest) {
        asterisk(&mut svg, p);
    }
    if let Some(p) = view.project(map.subsolar) {
        cross(&mut svg, p);
    }
    captions(&mut svg, eclipse);
    Ok(svg.finish())
}

/// The globe's disc, graticule and basemap.
fn globe(svg: &mut Svg, view: &Ortho, basemap: &Basemap) {
    svg.push(&format!(
        "<circle cx=\"{}\" cy=\"{}\" r=\"{RADIUS}\" fill=\"#ffffff\" stroke=\"#202020\" \
         stroke-width=\"1.2\"/>",
        CENTRE[0], CENTRE[1]
    ));
    svg.path(
        &view.polyline_d(&graticule(), 1.0),
        "fill=\"none\" stroke=\"#d4d4d4\" stroke-width=\"0.5\"",
    );
    svg.path(
        &view.polyline_d(&basemap.land, BASEMAP_STEP),
        "fill=\"none\" stroke=\"#202020\" stroke-width=\"0.6\"",
    );
    svg.path(
        &view.polyline_d(&basemap.borders, BASEMAP_STEP),
        "fill=\"none\" stroke=\"#909090\" stroke-width=\"0.4\"",
    );
}

/// Meridians and parallels every 30°.
fn graticule() -> Vec<Vec<LatLon>> {
    let meridians = (-6..6).map(|i| {
        (-45..=45)
            .map(|j| LatLon {
                lat: f64::from(j) * 2.0,
                lon: f64::from(i) * 30.0,
            })
            .collect()
    });
    let parallels = (-2..=2).map(|i| {
        (-90..=90)
            .map(|j| LatLon {
                lat: f64::from(i) * 30.0,
                lon: f64::from(j) * 2.0,
            })
            .collect()
    });
    meridians.chain(parallels).collect()
}

/// The night side, the penumbra's edge and the umbra at each frame.
fn moving(svg: &mut Svg, view: &Ortho, eclipse: &SolarEclipse) -> Result<(), EclipseError> {
    let (start, end) = eclipse.span();
    let mut night = Vec::new();
    let mut penumbra = Vec::new();
    let mut umbra = Vec::new();
    for &tt in ANIMATION.instants(start, end).iter().take(ANIMATION.frames) {
        let sun = overhead(eclipse.elements(tt)?.sun);
        let anti = LatLon {
            lat: -sun.lat,
            lon: sun.lon + 180.0,
        };
        night.push(view.hemisphere_d(anti).unwrap_or_else(|| EMPTY_D.into()));
        let outline = eclipse.shadow_outline(tt)?;
        penumbra.push(non_empty(view.polyline_d(&outline.penumbra, 1.0)));
        let fills: String = outline
            .umbra
            .iter()
            .filter_map(|u| view.polygon_d(u))
            .collect();
        umbra.push(non_empty(fills));
    }
    let animated = |values: &[String], style: &str| {
        format!(
            "<path d=\"{}\" {style}>{}</path>",
            values.first().map_or(EMPTY_D, String::as_str),
            ANIMATION.discrete("d", values)
        )
    };
    svg.push(&animated(
        &night,
        "fill=\"#303038\" fill-opacity=\"0.35\" stroke=\"none\"",
    ));
    svg.push(&animated(
        &penumbra,
        "fill=\"none\" stroke=\"#404040\" stroke-width=\"1.2\" stroke-dasharray=\"5 4\"",
    ));
    svg.push(&animated(
        &umbra,
        "fill=\"#0a0a14\" fill-opacity=\"0.7\" stroke=\"none\"",
    ));
    Ok(())
}

fn non_empty(d: String) -> String {
    if d.is_empty() { EMPTY_D.into() } else { d }
}

/// The Canon's curves in its colours; a partial eclipse has no path.
fn curves(svg: &mut Svg, view: &Ortho, eclipse: &SolarEclipse, map: &SolarMap) {
    let solid = |color: &str| format!("fill=\"none\" stroke=\"{color}\" stroke-width=\"1.8\"");
    let dashed = |color: &str| format!("{} stroke-dasharray=\"6 5\"", solid(color));
    let layers: [(&[Vec<LatLon>], String); 4] = [
        (&map.penumbral_limits, solid(PENUMBRA)),
        (&map.rise_set, solid(PENUMBRA)),
        (&map.max_rise_set, solid(MAX_RISE_SET)),
        (&map.half_magnitude, dashed(PENUMBRA)),
    ];
    for (layer, style) in layers {
        svg.path(&view.polyline_d(layer, 1.0), &style);
    }
    if eclipse.central {
        svg.path(&view.polyline_d(&map.umbral_limits, 1.0), &solid(UMBRA));
        svg.path(&view.polyline_d(&map.central_line, 1.0), &dashed(UMBRA));
    }
}

/// The Canon's mark of greatest eclipse.
fn asterisk(svg: &mut Svg, [x, y]: [f64; 2]) {
    let mut d = String::new();
    for i in 0..4 {
        let (s, c) = (f64::from(i) * std::f64::consts::FRAC_PI_4).sin_cos();
        d.push_str(&format!(
            "M{:.1} {:.1}L{:.1} {:.1}",
            x - 8.0 * c,
            y - 8.0 * s,
            x + 8.0 * c,
            y + 8.0 * s
        ));
    }
    svg.path(&d, &format!("stroke=\"{PENUMBRA}\" stroke-width=\"2\""));
    svg.push(&format!(
        "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"4\" fill=\"none\" stroke=\"{PENUMBRA}\" \
         stroke-width=\"2\"/>"
    ));
}

/// The Canon's mark of the subsolar point at greatest eclipse.
fn cross(svg: &mut Svg, [x, y]: [f64; 2]) {
    let d = format!(
        "M{:.1} {:.1}L{:.1} {:.1}M{:.1} {:.1}L{:.1} {:.1}",
        x - 7.0,
        y - 7.0,
        x + 7.0,
        y + 7.0,
        x - 7.0,
        y + 7.0,
        x + 7.0,
        y - 7.0
    );
    svg.path(&d, &format!("stroke=\"{PENUMBRA}\" stroke-width=\"2.5\""));
}

/// The title, the eclipse's figures, the legend and the clock of each frame.
fn captions(svg: &mut Svg, eclipse: &SolarEclipse) {
    let greatest = CalendarTime::from_jd(ut1(eclipse.greatest_tt));
    svg.text(
        CENTRE[0],
        36.0,
        &format!(
            "{} Solar Eclipse of {}",
            eclipse.kind.name(),
            greatest.date()
        ),
        "font-size=\"22\" font-weight=\"bold\" text-anchor=\"middle\"",
    );
    let mut figures = format!(
        "Greatest eclipse {} UT · Gamma {:.4} · Magnitude {:.4} · Saros {}",
        greatest.time(),
        eclipse.gamma,
        eclipse.magnitude,
        eclipse.saros
    );
    if let Some(seconds) = eclipse.central_duration_s {
        let seconds = seconds.round() as i64;
        figures.push_str(&format!(
            " · Duration {}m{:02}s",
            seconds / 60,
            seconds % 60
        ));
    }
    svg.text(
        CENTRE[0],
        62.0,
        &figures,
        "font-size=\"13\" text-anchor=\"middle\"",
    );
    let mut legend = vec![
        ("Penumbral limits", PENUMBRA),
        ("Maximum at sunrise and sunset", MAX_RISE_SET),
    ];
    if eclipse.central {
        legend.push(("Path and central line", UMBRA));
    }
    for (i, (label, color)) in legend.into_iter().enumerate() {
        svg.text(
            20.0,
            HEIGHT - 50.0 + 16.0 * i as f64,
            label,
            &format!("font-size=\"12\" fill=\"{color}\""),
        );
    }
    let (start, end) = eclipse.span();
    for (frame, tt) in ANIMATION
        .instants(start, end)
        .into_iter()
        .take(ANIMATION.frames)
        .enumerate()
    {
        let at = CalendarTime::from_jd(ut1(tt));
        svg.push(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"15\" text-anchor=\"end\" \
             visibility=\"hidden\">{} UT{}</text>",
            WIDTH - 20.0,
            HEIGHT - 18.0,
            at.time(),
            ANIMATION.only_during(frame)
        ));
    }
}

#[cfg(test)]
mod tests;
