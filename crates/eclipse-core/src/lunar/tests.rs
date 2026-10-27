use super::*;
use crate::time::CalendarTime;

fn march_2026() -> LunarEclipse {
    let jd = CalendarTime::jd_at_midnight(2026, 3, 3) + 11.58 / 24.0;
    LunarEclipse::near(jd).unwrap().unwrap()
}

#[test]
fn total_eclipse_of_2026_march_3() {
    // The Canon: gamma -0.3765, penumbral 2.1838, umbral 1.1507, 58.3 min of totality.
    let e = march_2026();
    assert_eq!(e.kind, LunarKind::Total);
    assert!((e.gamma + 0.3765).abs() < 0.001, "{}", e.gamma);
    assert!((e.umbral_magnitude - 1.1507).abs() < 0.002);
    let c = e.contacts;
    let total = (c.u3.unwrap() - c.u2.unwrap()) * 1440.0;
    assert!((total - 58.3).abs() < 0.3, "{total}");
    assert!(c.p1 < c.u1 && c.u1 < c.u2 && c.u3 < c.u4 && c.u4 < c.p4);
    assert_eq!(e.saros, 133);
}

#[test]
fn figure_geometry() {
    let e = march_2026();
    let [penumbra, umbra, moon] = e.radii(e.greatest_tt).unwrap();
    assert!(penumbra > umbra && umbra > moon && (14.0..18.0).contains(&moon));
    // At first umbral contact the Moon's limb touches the umbra.
    let u1 = e.contacts.u1.unwrap();
    let at = e.moon_in_shadow(u1).unwrap();
    let [_, umbra_then, moon_then] = e.radii(u1).unwrap();
    assert!((at.east.hypot(at.north) - umbra_then - moon_then).abs() < 0.01);
    // The shadow moves east along the ecliptic.
    let [east, north] = e.shadow_motion().unwrap();
    assert!(east > 0.8 && (east.hypot(north) - 1.0).abs() < 1e-9);
}

#[test]
fn visibility() {
    let e = march_2026();
    // Greatest eclipse at 11:34 TD: the Moon stands over the Pacific near 171° W.
    let below = e.sublunar(e.greatest_tt).unwrap();
    assert!(
        (below.lon + 171.0).abs() < 3.0 && below.lat.abs() < 10.0,
        "{below:?}"
    );
    assert_eq!(e.visible_share(below).unwrap(), 1.0);
    let antipode = LatLon {
        lat: -below.lat,
        lon: below.lon + 180.0,
    };
    assert_eq!(e.visible_share(antipode).unwrap(), 0.0);
}
