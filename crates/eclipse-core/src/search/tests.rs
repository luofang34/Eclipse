use super::*;

#[test]
fn the_eclipses_of_2026() {
    let found = eclipses_in_year(2026).unwrap();
    let summary: Vec<String> = found
        .iter()
        .map(|e| {
            let (kind, code) = match e {
                Eclipse::Solar(s) => ("solar", s.kind.code()),
                Eclipse::Lunar(l) => ("lunar", l.kind.code()),
            };
            format!(
                "{} {kind} {code}",
                CalendarTime::from_jd(e.greatest_tt()).iso_date()
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            "2026-02-17 solar A",
            "2026-03-03 lunar T",
            "2026-08-12 solar T",
            "2026-08-28 lunar P",
        ]
    );
}

#[test]
fn the_ends_of_the_range() {
    // Navigate covers 1850 to 2100: the year's eclipses inside it are still found.
    assert!(!eclipses_in_year(1850).unwrap().is_empty());
    assert!(!eclipses_in_year(2099).unwrap().is_empty());
}
