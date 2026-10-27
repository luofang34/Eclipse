//! Every lunar eclipse of NASA's Five Millennium Canon from 1850 to 2099 against what
//! eclipse-core computes.

use eclipse_core::{LunarEclipse, time::CalendarTime};

const CATALOGUE: &str = include_str!("fixtures/lunar_catalogue.csv");

#[cfg(test)]
fn jd(date: &str, time: &str) -> f64 {
    let d: Vec<i32> = date.split('-').map(|x| x.parse().unwrap()).collect();
    let t: Vec<f64> = time.split(':').map(|x| x.parse().unwrap()).collect();
    CalendarTime::jd_at_midnight(d[0], d[1] as u8, d[2] as u8)
        + (t[0] + t[1] / 60.0 + t[2] / 3600.0) / 24.0
}

/// The ways one catalogue row differs from the computed eclipse.
#[cfg(test)]
fn check_row(line: &str, worst: &mut [f64; 4]) -> Vec<String> {
    let row: Vec<&str> = line.split(',').collect();
    let (date, td) = (row[0], row[1]);
    let Some(e) = LunarEclipse::near(jd(date, td)).unwrap() else {
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
    let differences = [
        (e.gamma - num(5)).abs(),
        (e.penumbral_magnitude - num(6)).abs(),
        (e.umbral_magnitude - num(7)).abs(),
    ];
    for (w, d) in worst.iter_mut().zip(differences) {
        *w = w.max(d);
    }
    check(
        differences[0] < 0.0015,
        format!("gamma {:.4} vs {}", e.gamma, row[5]),
    );
    check(
        differences[1] < 0.003,
        format!("penumbral {:.4} vs {}", e.penumbral_magnitude, row[6]),
    );
    check(
        differences[2] < 0.003,
        format!("umbral {:.4} vs {}", e.umbral_magnitude, row[7]),
    );
    let c = e.contacts;
    for (column, (a, b)) in [(8, (c.p1, c.p4)), (9, (c.u1, c.u4)), (10, (c.u2, c.u3))] {
        if row[column].is_empty() {
            continue;
        }
        let minutes = a.zip(b).map_or(f64::NAN, |(a, b)| (b - a) * 1440.0);
        worst[3] = worst[3].max((minutes - num(column)).abs());
        // A phase that barely happens is a short chord across the shadow's edge, and its
        // length changes fast with the Moon's distance from that edge: a 0.002 difference
        // in magnitude changes a chord of D minutes by about 60 / D minutes.
        let tolerance = 0.5 + 60.0 / num(column);
        check(
            (minutes - num(column)).abs() <= tolerance,
            format!("phase {column} {minutes:.1} min vs {}", row[column]),
        );
    }
    failures
}

#[test]
fn every_catalogued_lunar_eclipse() {
    let mut failures = Vec::new();
    let mut worst = [0.0_f64; 4];
    for line in CATALOGUE.lines().skip(1) {
        failures.extend(check_row(line, &mut worst));
    }
    println!(
        "worst: gamma {:.5}, penumbral {:.5}, umbral {:.5}, phase {:.2} min",
        worst[0], worst[1], worst[2], worst[3]
    );
    assert!(
        failures.is_empty(),
        "{} mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
