//! Searching the years Navigate covers finds exactly the eclipses the NASA Five Millennium
//! Canon lists, with the same types: every year from 1800 to 2199, and with `long-range`
//! every seventh year from -1999 to 2999 besides.

use std::collections::BTreeMap;

use eclipse_core::{Eclipse, eclipses_in_year};

const SOLAR: &str = include_str!("fixtures/solar_catalogue.csv");
const LUNAR: &str = include_str!("fixtures/lunar_catalogue.csv");

// The search needs the fixture's years, not its instants.
#[allow(dead_code)]
#[path = "support/catalogue.rs"]
mod catalogue;

/// The years searched: all of 1800 to 2199, and a seventh of the rest.
#[cfg(test)]
fn searched(year: i32) -> bool {
    catalogue::covered(year) && ((1800..=2199).contains(&year) || year.rem_euclid(7) == 0)
}

/// Whether a catalogue row sits on a threshold its type turns on, as the Canon tests read
/// it: a solar eclipse whose axis grazes the limb, or a lunar one whose umbral magnitude is
/// within 0.003 of 0 or 1, or whose penumbral magnitude is within 0.003 of 0.
#[cfg(test)]
fn at_threshold(body: &str, row: &[&str]) -> bool {
    let num = |i: usize| row[i].parse::<f64>().unwrap_or(f64::NAN);
    match body {
        "solar" => num(9) == 0.0 && row[10].is_empty() && num(5).abs() < 1.03,
        _ => [(7, 1.0), (7, 0.0), (6, 0.0)]
            .iter()
            .any(|&(i, edge)| (num(i) - edge).abs() < 0.003),
    }
}

#[test]
fn every_year_finds_the_catalogued_eclipses() {
    let mut expected = BTreeMap::new();
    for (csv, body) in [(SOLAR, "solar"), (LUNAR, "lunar")] {
        for line in catalogue::rows(csv) {
            let row: Vec<&str> = line.split(',').collect();
            if !searched(catalogue::year(row[0])) {
                continue;
            }
            // Keyed by lunation: an eclipse the Canon's Moon times up to a minute apart
            // from Navigate's may fall on the other side of midnight.
            let key = format!("{} {body}", row[2]);
            expected.insert(key, (row[4].to_string(), at_threshold(body, &row)));
        }
    }
    let mut found = BTreeMap::new();
    for year in (-1999..=2999).filter(|&y| searched(y)) {
        for e in eclipses_in_year(year).unwrap() {
            // A graze within the magnitudes' tolerance of 0 may be found or not.
            let (body, code, graze, lunation) = match &e {
                Eclipse::Solar(s) => ("solar", s.kind.code(), s.magnitude < 0.003, s.lunation),
                Eclipse::Lunar(l) => (
                    "lunar",
                    l.kind.code(),
                    l.penumbral_magnitude < 0.003,
                    l.lunation,
                ),
            };
            found.insert(format!("{lunation} {body}"), (code.to_string(), graze));
        }
    }
    let mut wrong = Vec::new();
    for (key, (code, threshold)) in &expected {
        match found.get(key) {
            Some((ours, _)) if ours == code || *threshold => {}
            None if *threshold => {}
            other => wrong.push(format!("{key} {code}: found {other:?}")),
        }
    }
    let extra: Vec<_> = found
        .iter()
        .filter(|(key, (_, graze))| !graze && !expected.contains_key(*key))
        .map(|(key, _)| key)
        .collect();
    assert!(
        wrong.is_empty() && extra.is_empty(),
        "differ {wrong:?}\nextra {extra:?}"
    );
}
