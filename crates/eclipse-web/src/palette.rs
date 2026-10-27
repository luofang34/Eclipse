//! The map's colours, after the NASA Five Millennium Canon's maps.

use eframe::egui::Color32;

/// Penumbral limits, begin and end at sunrise and sunset, magnitude 0.5, markers.
pub(crate) const PENUMBRA: Color32 = Color32::from_rgb(0x12, 0x8a, 0x2a);
/// Maximum eclipse at sunrise and sunset; for a lunar eclipse, the Moon's horizon.
pub(crate) const MAX_RISE_SET: Color32 = Color32::from_rgb(0xe0, 0x1e, 0x84);
/// The path of totality or annularity and the central line.
pub(crate) const UMBRA: Color32 = Color32::from_rgb(0x1d, 0x3f, 0xd8);
/// The night side.
pub(crate) const NIGHT: Color32 = Color32::from_rgba_premultiplied(0x30, 0x30, 0x38, 0x70);
/// The umbra on the ground at the time shown.
pub(crate) const SHADOW: Color32 = Color32::from_rgba_premultiplied(0x0a, 0x0a, 0x14, 0xb0);
/// The penumbra's edge on the ground at the time shown.
pub(crate) const SHADOW_EDGE: Color32 = Color32::from_rgb(0x40, 0x40, 0x40);
/// The Earth's penumbra in the lunar figure.
pub(crate) const FIGURE_PENUMBRA: Color32 = Color32::from_rgb(0xe2, 0xe2, 0xe2);
/// The Earth's umbra in the lunar figure: the red of a totally eclipsed Moon.
pub(crate) const FIGURE_UMBRA: Color32 = Color32::from_rgb(0xa8, 0x16, 0x16);
