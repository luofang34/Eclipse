//! The basemap style: a white globe with coastlines and borders, as the NASA eclipse maps
//! draw it, from Natural Earth's 1:50m data.

use serde_json::{Value, json};

use crate::basemap::{BORDERS, LAND};

/// The basemap as a MapLibre style document.
pub(super) fn basemap() -> Result<Value, serde_json::Error> {
    let land: Value = serde_json::from_str(LAND)?;
    let borders: Value = serde_json::from_str(BORDERS)?;
    Ok(json!({
        "version": 8,
        "name": "Eclipse basemap",
        "center": [0.0, 20.0],
        "zoom": 0.5,
        "projection": {"type": "vertical-perspective"},
        "sources": {
            "land": {"type": "geojson", "data": land},
            "borders": {"type": "geojson", "data": borders},
        },
        "layers": [
            {"id": "ocean", "type": "background", "paint": {"background-color": "#ffffff"}},
            {"id": "coast", "type": "line", "source": "land",
             "paint": {"line-color": "#202020", "line-width": 0.8}},
            {"id": "borders", "type": "line", "source": "borders",
             "paint": {"line-color": "#e39a2d", "line-width": 0.8}},
        ],
    }))
}
