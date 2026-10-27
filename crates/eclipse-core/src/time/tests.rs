use super::*;

#[test]
fn calendar_round_trips() {
    let jd = CalendarTime::jd_at_midnight(2026, 8, 12) + (17.0 + 47.0 / 60.0 + 6.0 / 3600.0) / 24.0;
    let t = CalendarTime::from_jd(jd);
    assert_eq!(
        (t.year, t.month, t.day, t.hour, t.minute, t.second),
        (2026, 8, 12, 17, 47, 6)
    );
    assert_eq!(t.date(), "2026 Aug 12");
    assert_eq!(CalendarTime::jd_at_midnight(2000, 1, 1), 2_451_544.5);
}

#[test]
fn dates_before_the_reform_are_julian() {
    // Meeus, Astronomical Algorithms, chapter 7.
    assert_eq!(CalendarTime::jd_at_midnight(333, 1, 27) + 0.5, 1_842_713.0);
    assert_eq!(
        CalendarTime::jd_at_midnight(-1000, 7, 12) + 0.5,
        1_356_001.0
    );
    assert_eq!(CalendarTime::jd_at_midnight(-4712, 1, 1) + 0.5, 0.0);
    // The Julian 1582 October 4 is followed by the Gregorian October 15.
    let last_julian = CalendarTime::jd_at_midnight(1582, 10, 4);
    assert_eq!(
        CalendarTime::jd_at_midnight(1582, 10, 15),
        last_julian + 1.0
    );
    let next = CalendarTime::from_jd(last_julian + 1.0);
    assert_eq!((next.year, next.month, next.day), (1582, 10, 15));
}

#[test]
fn years_before_zero_read_as_the_catalogues_write_them() {
    let t = CalendarTime::from_jd(CalendarTime::jd_at_midnight(-584, 5, 28) + 0.8);
    assert_eq!((t.year, t.month, t.day), (-584, 5, 28));
    assert_eq!(t.date(), "-0584 May 28");
    assert_eq!(t.iso_date(), "-0584-05-28");
}
