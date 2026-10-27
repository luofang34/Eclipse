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
        // Bur-Sagale, -762 June 15; Thales, -584 May 28; Babylon, -135 April 15.
        (-34_156, 44),
        (-31_955, 57),
        (-26_403, 75),
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
        // Columbus's eclipse, 1504 March 1.
        (-6_133, 105),
    ] {
        assert_eq!(lunar_saros(lunation), saros, "lunar lunation {lunation}");
    }
}
