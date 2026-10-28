//! The three layers, each a GeoJSON feature collection.

/// Land polygons.
pub const LAND: &str = include_str!("../data/land.geojson");
/// The land's shores, as lines.
pub const COAST: &str = include_str!("../data/coast.geojson");
/// Boundaries between countries, as lines.
pub const BORDERS: &str = include_str!("../data/borders.geojson");
