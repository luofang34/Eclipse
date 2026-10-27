use super::*;

#[test]
fn catalogue_series() {
    // Lunation and series from the Canon's catalogues.
    for (lunation, saros) in [
        (218, 145),
        (300, 139),
        (329, 126),
        (341, 136),
        (564, 136),
        (1233, 136),
        (-1836, 143),
        (-997, 136),
    ] {
        assert_eq!(solar_saros(lunation), saros, "solar lunation {lunation}");
    }
    for (lunation, saros) in [
        (317, 128),
        (323, 133),
        (329, 138),
        (-1772, 134),
        (188, 132),
        (364, 130),
    ] {
        assert_eq!(lunar_saros(lunation), saros, "lunar lunation {lunation}");
    }
}
