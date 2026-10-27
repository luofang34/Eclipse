//! Searching every year from 1850 to 2099 finds exactly the eclipses the NASA Five
//! Millennium Canon lists, with the same types.

use std::collections::BTreeSet;

use eclipse_core::{Eclipse, eclipses_in_year, time::CalendarTime};

const SOLAR: &str = include_str!("fixtures/solar_catalogue.csv");
const LUNAR: &str = include_str!("fixtures/lunar_catalogue.csv");

#[test]
fn every_year_finds_the_catalogued_eclipses() {
    let mut expected = BTreeSet::new();
    for (catalogue, body) in [(SOLAR, "solar"), (LUNAR, "lunar")] {
        for line in catalogue.lines().skip(1) {
            let row: Vec<&str> = line.split(',').collect();
            expected.insert(format!("{} {body} {}", row[0], row[4]));
        }
    }
    let mut found = BTreeSet::new();
    for year in 1850..=2099 {
        for e in eclipses_in_year(year).unwrap() {
            let (body, code) = match &e {
                Eclipse::Solar(s) => ("solar", s.kind.code()),
                Eclipse::Lunar(l) => ("lunar", l.kind.code()),
            };
            let date = CalendarTime::from_jd(e.greatest_tt()).iso_date();
            found.insert(format!("{date} {body} {code}"));
        }
    }
    let missing: Vec<_> = expected.difference(&found).collect();
    let extra: Vec<_> = found.difference(&expected).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "missing {missing:?}\nextra {extra:?}"
    );
}
