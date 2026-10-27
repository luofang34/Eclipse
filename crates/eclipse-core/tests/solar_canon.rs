//! Every solar eclipse of NASA's Five Millennium Canon from 1850 to 2100, and the
//! published path of 2026 August 12, against what eclipse-core computes.

use eclipse_core::{
    SolarEclipse,
    geodesy::{LatLon, distance_km, lat_lon, surface},
    time::CalendarTime,
};

const CATALOGUE: &str = include_str!("fixtures/solar_catalogue.csv");
const PATH_AUGUST_2026: &str = include_str!("fixtures/path_2026_08_12.csv");
const PATH_FEBRUARY_2026: &str = include_str!("fixtures/path_2026_02_17.csv");

#[cfg(test)]
fn jd(date: &str, time: &str) -> f64 {
    let d: Vec<i32> = date.split('-').map(|x| x.parse().unwrap()).collect();
    let t: Vec<f64> = time.split(':').map(|x| x.parse().unwrap()).collect();
    let hours = t[0] + t[1] / 60.0 + t.get(2).copied().unwrap_or(0.0) / 3600.0;
    CalendarTime::jd_at_midnight(d[0], d[1] as u8, d[2] as u8) + hours / 24.0
}

#[cfg(test)]
fn rows(csv: &str) -> impl Iterator<Item = Vec<&str>> {
    csv.lines().skip(1).map(|line| line.split(',').collect())
}

#[test]
fn every_catalogued_solar_eclipse() {
    let mut failures = Vec::new();
    let mut worst = [0.0_f64; 5];
    for line in CATALOGUE.lines().skip(1) {
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
    let mut failures = Vec::new();
    let mut check = |ok: bool, what: String| {
        if !ok {
            failures.push(format!("{date} {}: {what}", row[4]));
        }
    };
    check(
        e.kind.code().to_string() == row[4],
        format!("type {:?}", e.kind),
    );
    check(
        e.lunation.to_string() == row[2],
        format!("lunation {}", e.lunation),
    );
    check(e.saros.to_string() == row[3], format!("saros {}", e.saros));
    let (dg, dm) = ((e.gamma - num(5)).abs(), (e.magnitude - num(6)).abs());
    worst[0] = worst[0].max(dg);
    worst[1] = worst[1].max(dm);
    check(dg < 0.0015, format!("gamma {:.4} vs {}", e.gamma, row[5]));
    let magnitude_tolerance = if e.central { 0.0015 } else { 0.003 };
    check(
        dm < magnitude_tolerance,
        format!("magnitude {:.4} vs {}", e.magnitude, row[6]),
    );
    // The Canon rounds positions and altitudes to whole degrees.
    check(
        (e.greatest.lat - num(7)).abs() <= 0.75,
        format!("lat {:.2} vs {}", e.greatest.lat, row[7]),
    );
    // A whole degree of longitude is short near the poles, and a later Delta T turns the
    // Earth less under the shadow, 360° a sidereal day.
    let dlon = (e.greatest.lon - num(8) + 540.0).rem_euclid(360.0) - 180.0;
    let rotation = (e.delta_t.seconds - num(12)).abs() * 360.0 / 86_164.1;
    let tolerance = 0.75 / e.greatest.lat.to_radians().cos() + rotation;
    check(
        dlon.abs() <= tolerance,
        format!("lon {:.2} vs {}", e.greatest.lon, row[8]),
    );
    check(
        (e.sun_altitude - num(9)).abs() <= 0.75,
        format!("altitude {:.2} vs {}", e.sun_altitude, row[9]),
    );
    // The Canon gives no width under 5 km to compare, and measures wide paths under a low
    // Sun across the path differently (see `path_of_2026_february_17`).
    if !row[10].is_empty() && num(10) >= 5.0 && e.gamma.abs() < 0.95 {
        let w = e.path_width_km.unwrap_or(f64::NAN);
        let error = (w - num(10)).abs();
        worst[2] = worst[2].max(error / num(10));
        check(
            error / num(10) < 0.03 || error < 3.0,
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
/// curves.
#[cfg(test)]
fn path_errors(table: &str, date: &str, td: &str) -> [f64; 2] {
    let e = SolarEclipse::near(jd(date, td)).unwrap().unwrap();
    let map = e.map().unwrap();
    let mut worst = [0.0_f64; 2];
    for row in rows(table) {
        let point = |i: usize| -> Option<LatLon> {
            Some(LatLon {
                lat: row[i].parse().ok()?,
                lon: row[i + 1].parse().ok()?,
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
    let worst = path_errors(PATH_AUGUST_2026, "2026-08-12", "17:47:06");
    assert!(worst[0] < 4.0 && worst[1] < 4.0, "{worst:?}");
}

/// A wide annular path under a low Sun. The limits agree with the published ones, though
/// the Canon's width at greatest eclipse, 616.6 km, is narrower than the distance across
/// the path between them.
#[test]
fn path_of_2026_february_17() {
    let worst = path_errors(PATH_FEBRUARY_2026, "2026-02-17", "12:13:06");
    assert!(worst[0] < 4.0 && worst[1] < 4.0, "{worst:?}");
}
