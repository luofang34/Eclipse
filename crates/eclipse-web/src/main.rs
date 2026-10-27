//! Solar and lunar eclipse maps in the browser, from Navigate's Sun and Moon, drawn on the
//! MapLibre fork's globe. The page is web-only; `trunk serve` builds and serves it.

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod basemap;
// Geometry the page uses, built natively too so its tests run.
#[cfg(target_arch = "wasm32")]
mod export;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod eye;
#[cfg(target_arch = "wasm32")]
mod figure;
#[cfg(target_arch = "wasm32")]
mod finder;
#[cfg(target_arch = "wasm32")]
mod globe;
#[cfg(target_arch = "wasm32")]
mod map;
#[cfg(target_arch = "wasm32")]
mod overlay;
#[cfg(target_arch = "wasm32")]
mod palette;
#[cfg(target_arch = "wasm32")]
mod panel;
#[cfg(target_arch = "wasm32")]
mod selection;
#[cfg(target_arch = "wasm32")]
mod sky;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}

/// The page runs only in a browser; a native build has nothing to start.
#[cfg(not(target_arch = "wasm32"))]
fn main() {}
