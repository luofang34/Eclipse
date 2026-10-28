//! Natural Earth's 1:50m land, coasts and borders, which the globe's style and the SVG export
//! draw.

use eclipse_core::geodesy::LatLon;
use serde_json::Value;

pub(crate) use natural_earth_50m::{BORDERS, COAST, LAND};

/// The coasts and borders as lines, for the SVG export.
pub(crate) fn lines() -> Result<eclipse_svg::Basemap, serde_json::Error> {
    Ok(eclipse_svg::Basemap {
        land: rings(&serde_json::from_str(COAST)?),
        borders: rings(&serde_json::from_str(BORDERS)?),
    })
}

/// Every run of positions in a GeoJSON value: the rings of polygons and the lines of line
/// strings, whatever features and collections hold them.
fn rings(value: &Value) -> Vec<Vec<LatLon>> {
    let mut found = Vec::new();
    collect(value, &mut found);
    found
}

fn collect(value: &Value, found: &mut Vec<Vec<LatLon>>) {
    match value {
        Value::Object(object) => object.values().for_each(|v| collect(v, found)),
        Value::Array(items) if items.first().is_some_and(is_position) => {
            found.push(items.iter().filter_map(position).collect());
        }
        Value::Array(items) => items.iter().for_each(|v| collect(v, found)),
        _ => {}
    }
}

fn is_position(value: &Value) -> bool {
    position(value).is_some()
}

fn position(value: &Value) -> Option<LatLon> {
    let pair = value.as_array()?;
    Some(LatLon {
        lon: pair.first()?.as_f64()?,
        lat: pair.get(1)?.as_f64()?,
    })
}
