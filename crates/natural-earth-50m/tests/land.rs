//! The land is whole: every continent's polygons, each ring closed.

use natural_earth_50m::LAND;

#[test]
fn the_land_rings_are_closed() {
    let doc: serde_json::Value = serde_json::from_str(LAND).unwrap();
    let mut rings = 0;
    for feature in doc["features"].as_array().unwrap() {
        let geometry = &feature["geometry"];
        let polygons = match geometry["type"].as_str().unwrap() {
            "Polygon" => vec![&geometry["coordinates"]],
            _ => geometry["coordinates"].as_array().unwrap().iter().collect(),
        };
        for polygon in polygons {
            for ring in polygon.as_array().unwrap() {
                let ring = ring.as_array().unwrap();
                assert_eq!(ring.first(), ring.last());
                rings += 1;
            }
        }
    }
    assert!(rings > 1_000, "{rings} rings");
}
