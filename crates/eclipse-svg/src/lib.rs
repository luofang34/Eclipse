//! Animated SVG maps of eclipses: a solar eclipse's Canon curves on an orthographic globe
//! with the shadow and the night side moving across it, and a lunar eclipse's figure with
//! the Moon moving through the Earth's shadow. The animation is SMIL, which browsers play
//! from the file alone.

mod document;
mod lunar;
mod ortho;
mod solar;

pub use lunar::lunar_svg;
pub use ortho::Ortho;
pub use solar::{Basemap, solar_svg};
