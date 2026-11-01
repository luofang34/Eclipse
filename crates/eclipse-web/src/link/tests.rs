use super::{OVERVIEW_S, OpeningFlight, PageLink};

#[test]
fn a_date_alone_stays_on_the_globe() {
    let link = PageLink::parse("?date=2027-02-06");
    assert_eq!(link.date.as_deref(), Some("2027-02-06"));
    assert!(!link.stand);
}

#[test]
fn stand_greatest_asks_for_the_flight_in_either_order() {
    for search in [
        "?date=2027-02-06&stand=greatest",
        "?stand=greatest&date=2027-02-06",
    ] {
        let link = PageLink::parse(search);
        assert_eq!(link.date.as_deref(), Some("2027-02-06"));
        assert!(link.stand);
    }
}

#[test]
fn other_stand_values_and_malformed_dates_are_ignored() {
    assert!(!PageLink::parse("?date=2027-02-06&stand=here").stand);
    assert!(!PageLink::parse("?date=2027-02-06&stand").stand);
    assert_eq!(PageLink::parse("?date=2027").date, None);
}

#[test]
fn a_date_before_year_zero_keeps_its_sign() {
    let link = PageLink::parse("?date=-0584-05-28");
    assert_eq!(link.date.as_deref(), Some("-0584-05-28"));
}

#[test]
fn a_link_survives_writing_it_out_and_reading_it_back() {
    for link in [
        PageLink::default(),
        PageLink {
            date: Some("2027-02-06".to_owned()),
            stand: false,
            embed: false,
        },
        PageLink {
            date: Some("2027-02-06".to_owned()),
            stand: true,
            embed: true,
        },
    ] {
        assert_eq!(PageLink::parse(&link.query()), link);
    }
}

#[test]
fn the_overview_runs_from_the_first_frame_asked_about() {
    let mut flight = OpeningFlight::default();
    assert_eq!(flight.remaining(100.0), Some(OVERVIEW_S));
    assert_eq!(flight.remaining(100.5), Some(OVERVIEW_S - 0.5));
    assert_eq!(flight.remaining(100.0 + OVERVIEW_S), None);
}

#[test]
fn only_embed_1_frames_the_page() {
    assert!(PageLink::parse("?date=2027-02-06&embed=1").embed);
    assert!(!PageLink::parse("?date=2027-02-06&embed=0").embed);
    assert!(!PageLink::parse("?date=2027-02-06").embed);
}
