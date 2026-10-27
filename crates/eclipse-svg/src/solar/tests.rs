use super::*;

const AUGUST_2026: f64 = 2_461_265.25;

fn svg_of(jd: f64, basemap: &Basemap) -> String {
    let eclipse = SolarEclipse::near(jd).unwrap().unwrap();
    solar_svg(&eclipse, &eclipse.map().unwrap(), basemap).unwrap()
}

#[test]
fn the_map_is_well_formed_and_animates_every_frame() {
    let svg = svg_of(AUGUST_2026, &Basemap::default());
    let document = roxmltree::Document::parse(&svg).unwrap();
    let animations: Vec<_> = document
        .descendants()
        .filter(|n| n.has_tag_name("animate"))
        .collect();
    let shapes = animations
        .iter()
        .filter(|a| a.attribute("attributeName") == Some("d"))
        .count();
    assert_eq!(shapes, 3, "night, penumbra and umbra");
    for a in animations
        .iter()
        .filter(|a| a.attribute("attributeName") == Some("d"))
    {
        let values = a.attribute("values").unwrap().split(';').count();
        assert_eq!(values, ANIMATION.frames);
    }
    let clocks = animations
        .iter()
        .filter(|a| a.attribute("attributeName") == Some("visibility"))
        .count();
    assert_eq!(clocks, ANIMATION.frames);
    assert!(svg.contains("Total Solar Eclipse"));
}

#[test]
fn the_umbra_shows_in_some_frames_and_not_others() {
    let svg = svg_of(AUGUST_2026, &Basemap::default());
    let document = roxmltree::Document::parse(&svg).unwrap();
    let umbra = document
        .descendants()
        .filter(|n| n.has_tag_name("animate"))
        .filter(|a| a.attribute("attributeName") == Some("d"))
        .nth(2)
        .unwrap();
    let values: Vec<&str> = umbra.attribute("values").unwrap().split(';').collect();
    let shown = values.iter().filter(|v| **v != EMPTY_D).count();
    assert!(
        shown > 10 && shown < values.len(),
        "{shown} of {}",
        values.len()
    );
}

#[test]
fn a_dense_coastline_is_thinned_to_what_the_page_shows() {
    let dense: Vec<LatLon> = (0..=20_000)
        .map(|i| LatLon {
            lat: 45.0,
            lon: -20.0 + f64::from(i) * 0.001,
        })
        .collect();
    let basemap = Basemap {
        land: vec![dense],
        borders: Vec::new(),
    };
    let with = svg_of(AUGUST_2026, &basemap).len();
    let without = svg_of(AUGUST_2026, &Basemap::default()).len();
    // 20° of latitude circle is about 100 points at the page's scale, not 20 000.
    assert!(with - without < 4_000, "{} bytes", with - without);
}
