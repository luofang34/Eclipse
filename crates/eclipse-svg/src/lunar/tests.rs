use super::*;

/// 2026 March 3, a total lunar eclipse.
const MARCH_2026: f64 = 2_461_102.98;

#[test]
fn the_figure_is_well_formed_and_moves_the_moon() {
    let eclipse = LunarEclipse::near(MARCH_2026).unwrap().unwrap();
    let svg = lunar_svg(&eclipse).unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    let moving: Vec<&str> = document
        .descendants()
        .filter(|n| n.has_tag_name("animate"))
        .filter_map(|a| a.attribute("attributeName"))
        .filter(|name| *name == "cx" || *name == "cy")
        .collect();
    assert_eq!(moving, ["cx", "cy"]);
    for label in ["P1", "U1", "U2", "U3", "U4", "P4"] {
        assert!(svg.contains(&format!(">{label}</text>")), "{label}");
    }
    assert!(svg.contains("Total Lunar Eclipse"));
}
