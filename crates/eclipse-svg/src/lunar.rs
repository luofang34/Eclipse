//! A lunar eclipse's figure as the Canon draws it, north up and east to the left as the sky
//! is seen: the Earth's penumbra and umbra, the Moon at each contact, its path and the
//! ecliptic, with the Moon moving along its path.

use eclipse_core::{
    EclipseError, LunarEclipse,
    time::{CalendarTime, ut1},
};

use crate::document::{Animation, Svg};

const SIZE: f64 = 640.0;
const TOP: f64 = 80.0;
const ANIMATION: Animation = Animation {
    frames: 120,
    seconds: 20.0,
};

/// Where on the page a point given in arcminutes east and north of the shadow's centre is.
#[derive(Clone, Copy, Debug)]
struct Sky {
    centre: [f64; 2],
    scale: f64,
}

impl Sky {
    fn at(&self, east: f64, north: f64) -> [f64; 2] {
        // East is to the left on the sky.
        [
            self.centre[0] - east * self.scale,
            self.centre[1] - north * self.scale,
        ]
    }
}

/// The animated SVG figure of `eclipse`.
pub fn lunar_svg(eclipse: &LunarEclipse) -> Result<String, EclipseError> {
    let [penumbra, umbra, moon] = eclipse.radii(eclipse.greatest_tt)?;
    let places = contacts(eclipse)?;
    let reach = places
        .iter()
        .map(|(_, east, north)| east.hypot(*north) + moon)
        .fold(penumbra, f64::max);
    let sky = Sky {
        centre: [SIZE / 2.0, TOP + SIZE / 2.0],
        scale: 0.46 * SIZE / reach,
    };
    let mut svg = Svg::new(SIZE, SIZE + TOP + 30.0);
    let [cx, cy] = sky.centre;
    svg.push(&format!(
        "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{:.1}\" fill=\"#e2e2e2\"/>\
         <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{:.1}\" fill=\"#a81616\"/>",
        penumbra * sky.scale,
        umbra * sky.scale
    ));
    let [east, north] = eclipse.shadow_motion()?;
    let [a, b] = [
        sky.at(-east * reach, -north * reach),
        sky.at(east * reach, north * reach),
    ];
    svg.path(
        &format!("M{:.1} {:.1}L{:.1} {:.1}", a[0], a[1], b[0], b[1]),
        "stroke=\"#000000\" stroke-dasharray=\"5 4\"",
    );
    let radius = moon * sky.scale;
    let ends = places.first().zip(places.last());
    if let Some(((_, e0, n0), (_, e1, n1))) = ends {
        // The path runs a Moon's radius past the first and last contacts.
        let length = (e1 - e0).hypot(n1 - n0).max(1e-9);
        let (de, dn) = ((e1 - e0) / length * moon, (n1 - n0) / length * moon);
        let [a, b] = [sky.at(e0 - de, n0 - dn), sky.at(e1 + de, n1 + dn)];
        svg.path(
            &format!("M{:.1} {:.1}L{:.1} {:.1}", a[0], a[1], b[0], b[1]),
            "stroke=\"#000000\"",
        );
    }
    for (name, east, north) in &places {
        let [x, y] = sky.at(*east, *north);
        svg.push(&format!(
            "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"{radius:.1}\" fill=\"none\" \
             stroke=\"#000000\"/>"
        ));
        svg.text(
            x,
            y + radius + 12.0,
            name,
            "font-size=\"11\" text-anchor=\"middle\"",
        );
    }
    moving_moon(&mut svg, eclipse, sky, radius)?;
    captions(&mut svg, eclipse);
    Ok(svg.finish())
}

/// The Moon at each contact and at greatest eclipse, which is unnamed, in arcminutes.
fn contacts(eclipse: &LunarEclipse) -> Result<Vec<(&'static str, f64, f64)>, EclipseError> {
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
    .map(|(name, tt)| {
        let at = eclipse.moon_in_shadow(tt)?;
        Ok((name, at.east, at.north))
    })
    .collect()
}

/// The Moon moving from first to last penumbral contact, with the clock of each frame.
fn moving_moon(
    svg: &mut Svg,
    eclipse: &LunarEclipse,
    sky: Sky,
    radius: f64,
) -> Result<(), EclipseError> {
    let (start, end) = match (eclipse.contacts.p1, eclipse.contacts.p4) {
        (Some(p1), Some(p4)) => (p1, p4),
        _ => eclipse.span(),
    };
    let instants = ANIMATION.instants(start, end);
    let mut xs = Vec::with_capacity(instants.len());
    let mut ys = Vec::with_capacity(instants.len());
    for &tt in &instants {
        let at = eclipse.moon_in_shadow(tt)?;
        let [x, y] = sky.at(at.east, at.north);
        xs.push(x);
        ys.push(y);
    }
    svg.push(&format!(
        "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{radius:.1}\" fill=\"#f0e8c8\" \
         fill-opacity=\"0.6\" stroke=\"#806010\" stroke-width=\"1.5\">{}{}</circle>",
        xs.first().copied().unwrap_or_default(),
        ys.first().copied().unwrap_or_default(),
        ANIMATION.linear("cx", &xs),
        ANIMATION.linear("cy", &ys)
    ));
    for (frame, tt) in instants.into_iter().take(ANIMATION.frames).enumerate() {
        svg.push(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"15\" text-anchor=\"end\" \
             visibility=\"hidden\">{} UT{}</text>",
            SIZE - 16.0,
            SIZE + TOP + 18.0,
            CalendarTime::from_jd(ut1(tt)).time(),
            ANIMATION.only_during(frame)
        ));
    }
    Ok(())
}

/// The title and the eclipse's figures.
fn captions(svg: &mut Svg, eclipse: &LunarEclipse) {
    let greatest = CalendarTime::from_jd(ut1(eclipse.greatest_tt));
    svg.text(
        SIZE / 2.0,
        34.0,
        &format!(
            "{} Lunar Eclipse of {}",
            eclipse.kind.name(),
            greatest.date()
        ),
        "font-size=\"22\" font-weight=\"bold\" text-anchor=\"middle\"",
    );
    svg.text(
        SIZE / 2.0,
        60.0,
        &format!(
            "Greatest eclipse {} UT · Umbral magnitude {:.4} · Penumbral magnitude {:.4} · \
             Saros {}",
            greatest.time(),
            eclipse.umbral_magnitude,
            eclipse.penumbral_magnitude,
            eclipse.saros
        ),
        "font-size=\"12\" text-anchor=\"middle\"",
    );
    svg.text(
        16.0,
        SIZE + TOP + 18.0,
        "N up, E left",
        "font-size=\"11\" fill=\"#606060\"",
    );
}

#[cfg(test)]
mod tests;
