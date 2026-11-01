//! The eclipses the book frames: each `data-query` date in its chapters must open the eclipse
//! it names, since the page silently shows another for a date with none. Dates beyond the
//! default Moon's years are checked with `long-range`.

use std::{fs, path::Path};

use eclipse_core::{eclipses_in_year, time::CalendarTime};

const FIRST_YEAR: i32 = 1800;
const LAST_YEAR: i32 = 2199;
const MARKER: &str = "data-query=\"date=";

/// The dates after `MARKER` in `chapter`.
fn dates(chapter: &str) -> Vec<&str> {
    chapter
        .split(MARKER)
        .skip(1)
        .filter_map(|rest| rest.split(['&', '"']).next())
        .collect()
}

#[test]
fn every_framed_date_names_an_eclipse() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../book/src");
    let mut checked = 0;
    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        for date in dates(&text) {
            let digits = date.trim_start_matches('-').get(..4).unwrap();
            let year: i32 = digits.parse().unwrap();
            let year = if date.starts_with('-') { -year } else { year };
            if !cfg!(feature = "long-range") && !(FIRST_YEAR..=LAST_YEAR).contains(&year) {
                continue;
            }
            let found = eclipses_in_year(year)
                .unwrap()
                .iter()
                .any(|eclipse| CalendarTime::from_jd(eclipse.greatest_tt()).iso_date() == date);
            assert!(found, "{}: no eclipse on {date}", path.display());
            checked += 1;
        }
    }
    assert!(checked > 0, "no framed dates in {}", source.display());
}
