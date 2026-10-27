//! Every solar eclipse of NASA's Five Millennium Canon in the years Navigate covers, 1800
//! to 2199 or with `long-range` -1999 to 2999, and the published paths of 2026, against
//! what eclipse-core computes.

use eclipse_core::{
    SolarEclipse,
    geodesy::{LatLon, distance_km, lat_lon, surface},
};

const CATALOGUE: &str = include_str!("fixtures/solar_catalogue.csv");
const PATH_AUGUST_2026: &str = include_str!("fixtures/path_2026_08_12.csv");
const PATH_FEBRUARY_2026: &str = include_str!("fixtures/path_2026_02_17.csv");

#[path = "support/catalogue.rs"]
mod catalogue;
use catalogue::jd;

#[cfg(test)]
fn rows(csv: &str) -> impl Iterator<Item = Vec<&str>> {
    csv.lines().skip(1).map(|line| line.split(',').collect())
}

#[test]
fn every_catalogued_solar_eclipse() {
    let mut failures = Vec::new();
    let mut worst = [0.0_f64; 5];
    for line in catalogue::rows(CATALOGUE) {
        failures.extend(check_row(line, &mut worst));
    }
    println!(
        "worst: gamma {:.5}, magnitude {:.5}, width {:.4}, duration {:.1} s, time {:.1} s",
        worst[0], worst[1], worst[2], worst[3], worst[4]
    );
    assert!(
        failures.is_empty(),
        "{} mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The ways one catalogue row differs from the computed eclipse, recording the largest
/// differences in `worst`: gamma, magnitude, relative width, duration, time.
#[cfg(test)]
fn check_row(line: &str, worst: &mut [f64; 5]) -> Vec<String> {
    let row: Vec<&str> = line.split(',').collect();
    let (date, td) = (row[0], row[1]);
    let Some(e) = SolarEclipse::near(jd(date, td)).unwrap() else {
        return vec![format!("{date}: not found")];
    };
    let num = |i: usize| row[i].parse::<f64>().unwrap();
    // With greatest eclipse at the Sun's altitude 0, no path and gamma within 1.03, the
    // shadow's axis passes within an umbra's breadth of the Earth's limb: whether the umbra
    // touches the Earth is then a matter of kilometres, and the type and magnitude may go
    // either way.
    let grazing = num(9) == 0.0 && row[10].is_empty() && num(5).abs() < 1.03;
    // Under a low Sun the point of greatest eclipse slides fast along the limb.
    let low = num(9) < 10.0;
    let mut failures = Vec::new();
    let mut check = |ok: bool, what: String| {
        if !ok {
            failures.push(format!("{date} {}: {what}", row[4]));
        }
    };
    let types = [e.kind.code().to_string(), row[4].to_string()];
    let borderline = match [types[0].as_str(), types[1].as_str()] {
        ["T" | "A" | "H", "P"] | ["P", "T" | "A" | "H"] => grazing,
        ["T", "H"] | ["H", "T"] | ["A", "H"] | ["H", "A"] => central_ends_at_threshold(&e),
        _ => false,
    };
    check(
        types[0] == types[1] || borderline,
        format!("type {:?}", e.kind),
    );
    check(
        e.lunation.to_string() == row[2],
        format!("lunation {}", e.lunation),
    );
    check(e.saros.to_string() == row[3], format!("saros {}", e.saros));
    let (dg, dm) = ((e.gamma - num(5)).abs(), (e.magnitude - num(6)).abs());
    worst[0] = worst[0].max(dg);
    check(dg < 0.0015, format!("gamma {:.4} vs {}", e.gamma, row[5]));
    if !grazing {
        worst[1] = worst[1].max(dm);
        let magnitude_tolerance = if e.central { 0.0015 } else { 0.003 };
        check(
            dm < magnitude_tolerance,
            format!("magnitude {:.4} vs {}", e.magnitude, row[6]),
        );
    }
    // The Canon rounds positions and altitudes to whole degrees.
    let degrees = if low { 1.5 } else { 0.75 };
    check(
        (e.greatest.lat - num(7)).abs() <= degrees,
        format!("lat {:.2} vs {}", e.greatest.lat, row[7]),
    );
    // A whole degree of longitude is short near the poles, and a later Delta T turns the
    // Earth less under the shadow, 360° a sidereal day.
    let dlon = (e.greatest.lon - num(8) + 540.0).rem_euclid(360.0) - 180.0;
    let rotation = (e.delta_t.seconds - num(12)).abs() * 360.0 / 86_164.1;
    let tolerance = degrees / e.greatest.lat.to_radians().cos() + rotation;
    check(
        dlon.abs() <= tolerance,
        format!("lon {:.2} vs {}", e.greatest.lon, row[8]),
    );
    check(
        (e.sun_altitude - num(9)).abs() <= degrees,
        format!("altitude {:.2} vs {}", e.sun_altitude, row[9]),
    );
    // The Canon gives no width under 5 km to compare. It measures a path under a low Sun
    // more narrowly than the ground between its limits (see `path_of_2026_february_17`),
    // and the more so the wider the path's footprint, its width over the sine of the Sun's
    // altitude: 2 % for each 1000 km of that, and 3 % at least. Past gamma 0.95, where the
    // axis nears the limb, the two part by up to a third and are not compared.
    if !row[10].is_empty() && num(10) >= 5.0 && e.gamma.abs() < 0.95 {
        let w = e.path_width_km.unwrap_or(f64::NAN);
        let error = (w - num(10)).abs();
        worst[2] = worst[2].max(error / num(10));
        let footprint = num(10) / num(9).to_radians().sin();
        let tolerance = (0.02 * footprint / 1000.0).max(0.03);
        check(
            error / num(10) < tolerance || error < 3.0,
            format!("width {w:.1} vs {}", row[10]),
        );
    }
    if !row[11].is_empty() {
        let d = e.central_duration_s.unwrap_or(f64::NAN);
        worst[3] = worst[3].max((d - num(11)).abs());
        check(
            (d - num(11)).abs() <= 4.0,
            format!("duration {d:.1} vs {}", row[11]),
        );
    }
    worst[4] = worst[4].max((e.greatest_tt - jd(date, td)).abs() * 86_400.0);
    failures
}

/// Whether the umbra's radius at either end of the central line, where the axis meets the
/// Earth's limb, is within the elements' accuracy (0.0002 Earth radii) of zero, so that the
/// end may read total or annular.
#[cfg(test)]
fn central_ends_at_threshold(e: &SolarEclipse) -> bool {
    let (start, end) = e.span();
    let mut ends: Vec<f64> = Vec::new();
    let mut previous = false;
    let mut t = start;
    while t <= end {
        let el = e.elements(t).unwrap();
        let on = el.surface(el.x, el.y).map(|p| el.shadow(p).l2);
        if let Some(l2) = on {
            if !previous {
                ends.push(l2);
            }
            ends.truncate(1);
            ends.push(l2);
        }
        previous = on.is_some();
        t += 1.0 / 86_400.0;
    }
    ends.iter().any(|l2| l2.abs() < 0.0002)
}

/// The distance in km from `p` to the nearest of `curves`, sampling each segment.
#[cfg(test)]
fn distance_to(p: LatLon, curves: &[Vec<LatLon>]) -> f64 {
    let mut best = f64::INFINITY;
    for curve in curves {
        for pair in curve.windows(2) {
            let (a, b) = (surface(pair[0]), surface(pair[1]));
            for k in 0..=20 {
                let f = f64::from(k) / 20.0;
                let q = [0, 1, 2].map(|i| a[i] + f * (b[i] - a[i]));
                best = best.min(distance_km(p, lat_lon(q)));
            }
        }
    }
    best
}

/// The worst distances in km from the published central line and limits to the computed
/// curves. The table was computed with a Delta T of `published_delta_t` seconds; its points
/// are turned with the Earth to the Delta T used here, so only the shadow is compared.
#[cfg(test)]
fn path_errors(table: &str, date: &str, td: &str, published_delta_t: f64) -> [f64; 2] {
    // The Earth turns 15.041 arcseconds in a second.
    const DEGREES_PER_SECOND: f64 = 15.041 / 3600.0;
    let e = SolarEclipse::near(jd(date, td)).unwrap().unwrap();
    let map = e.map().unwrap();
    // A larger Delta T puts the same TT instant at an earlier UT1, with the Earth turned
    // less and the shadow falling further east.
    let turn = (e.delta_t.seconds - published_delta_t) * DEGREES_PER_SECOND;
    let mut worst = [0.0_f64; 2];
    for row in rows(table) {
        let point = |i: usize| -> Option<LatLon> {
            Some(LatLon {
                lat: row[i].parse().ok()?,
                lon: row[i + 1].parse::<f64>().ok()? + turn,
            })
        };
        if let Some(central) = point(5) {
            worst[0] = worst[0].max(distance_to(central, &map.central_line));
        }
        for limit in [point(1), point(3)].into_iter().flatten() {
            worst[1] = worst[1].max(distance_to(limit, &map.umbral_limits));
        }
    }
    println!(
        "{date}: central line {:.2} km, limits {:.2} km",
        worst[0], worst[1]
    );
    worst
}

#[test]
fn path_of_2026_august_12() {
    let worst = path_errors(PATH_AUGUST_2026, "2026-08-12", "17:47:06", 71.4);
    assert!(worst[0] < 4.0 && worst[1] < 4.0, "{worst:?}");
}

/// A wide annular path under a Sun 12° up, where the ground magnifies a shift of the shadow
/// about fivefold: the limits agree with the published ones to 6 km, from elements that
/// agree to 0.0001 Earth radii with `long-range`. The Canon's width at greatest eclipse,
/// 616.6 km, is narrower than the distance across the path between them.
#[test]
fn path_of_2026_february_17() {
    let worst = path_errors(PATH_FEBRUARY_2026, "2026-02-17", "12:13:06", 71.2);
    assert!(worst[0] < 6.0 && worst[1] < 6.0, "{worst:?}");
}

/// NASA's polynomial Besselian elements x, y, d, l1 and l2 for the eclipses of 2026, at t0
/// TT: each row holds a coefficient of t^0 to t^3, t in hours (eclipse.gsfc.nasa.gov
/// SEbeselm pages, from VSOP87 and ELP2000-85).
#[cfg(test)]
const ELEMENTS_2026: [(&str, &str, [[f64; 4]; 5]); 2] = [
    (
        "2026-02-17",
        "12:00:00",
        [
            [0.322033, 0.4827116, -0.0000336, -0.0000049],
            [-0.926964, 0.2355386, 0.0001168, -0.0000033],
            [-11.87930, 0.014049, 0.000002, 0.0],
            [0.557719, -0.0001181, -0.0000111, 0.0],
            [0.011523, -0.0001175, -0.0000111, 0.0],
        ],
    ),
    (
        "2026-08-12",
        "18:00:00",
        [
            [0.475593, 0.5189288, -0.0000773, -0.0000088],
            [0.771161, -0.2301664, -0.0001245, 0.0000037],
            [14.79667, -0.012065, -0.000003, 0.0],
            [0.537954, 0.0000940, -0.0000121, 0.0],
            [-0.008142, 0.0000935, -0.0000121, 0.0],
        ],
    ),
];

/// The shadow's geometry, free of Delta T, against NASA's elements over four hours: within
/// 0.0015 Earth radii (10 km) from the default Moon, 12" from DE421, and within 0.0002 from
/// the `long-range` one, 1" from DE421 and of the same ELP2000 lineage as NASA's.
#[test]
fn besselian_elements_of_2026() {
    let bound = if cfg!(feature = "long-range") {
        0.0002
    } else {
        0.0015
    };
    for (date, t0, published) in ELEMENTS_2026 {
        let e = SolarEclipse::near(jd(date, t0)).unwrap().unwrap();
        for hours in [-2.0, 0.0, 2.0] {
            let el = e.elements(jd(date, t0) + hours / 24.0).unwrap();
            let d = el.axis[2].asin().to_degrees();
            let ours = [el.x, el.y, d, el.l1, el.l2];
            for (i, (value, c)) in ours.iter().zip(published).enumerate() {
                let theirs = c[0] + hours * (c[1] + hours * (c[2] + hours * c[3]));
                // d is in degrees, as an angle of about `bound` radians.
                let limit = if i == 2 { bound * 57.3 } else { bound };
                assert!(
                    (value - theirs).abs() < limit,
                    "{date} {hours:+} h element {i}: {value:.6} vs {theirs:.6}"
                );
            }
        }
    }
}
