use super::*;
use navigate_celestial::civil_time;

#[test]
fn civil_epoch_maps_back_to_tt_on_both_sides_of_1972() {
    for (year, month, day) in [
        (1851, 7, 28),
        (1919, 5, 29),
        (1971, 12, 31),
        (1972, 1, 2),
        (2026, 8, 12),
        (2099, 9, 14),
    ] {
        let jd = CalendarTime::jd_at_midnight(year, month, day) + 0.3;
        let back = civil_time(civil_epoch(jd)).terrestrial;
        let error = (back.to_jde_tt_days() - jd) * SECONDS_PER_DAY;
        assert!(error.abs() < 1e-3, "{year}-{month}-{day}: {error} s");
    }
}

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
