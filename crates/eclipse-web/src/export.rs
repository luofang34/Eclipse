//! The chosen eclipse as an animated SVG file the browser saves.

use wasm_bindgen::{JsCast, JsValue, closure::Closure};

use crate::selection::Selected;

/// How long the saved file's blob is kept for the browser to read.
const RELEASE_AFTER_MS: i32 = 60_000;

/// Writes the SVG of `selected` and has the browser save it.
pub(crate) fn save(selected: &Selected) -> Result<(), String> {
    let kind = match selected {
        Selected::Solar { .. } => "solar",
        Selected::Lunar { .. } => "lunar",
    };
    let date = eclipse_core::time::CalendarTime::from_jd(selected.greatest_tt()).iso_date();
    let name = format!("{kind}-eclipse-{date}.svg");
    let svg = match selected {
        Selected::Solar { eclipse, map } => {
            let basemap = crate::basemap::lines().map_err(|e| format!("basemap: {e}"))?;
            eclipse_svg::solar_svg(eclipse, map, &basemap)
        }
        Selected::Lunar { eclipse, .. } => eclipse_svg::lunar_svg(eclipse),
    }
    .map_err(|e| e.to_string())?;
    download(&name, &svg).map_err(|e| format!("download refused: {e:?}"))
}

/// Offers `text` as a file named `name`, through a link to a blob the page clicks.
fn download(name: &str, text: &str) -> Result<(), JsValue> {
    let parts = js_sys::Array::of1(&JsValue::from_str(text));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("image/svg+xml");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let link: web_sys::HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    link.set_href(&url);
    link.set_download(name);
    link.click();
    // The browser reads the blob after the click returns, so it is let go only later.
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let release = Closure::once_into_js(move || {
        if let Err(error) = web_sys::Url::revoke_object_url(&url) {
            tracing::warn!(?error, "saved file's blob kept");
        }
    });
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        release.unchecked_ref(),
        RELEASE_AFTER_MS,
    )?;
    Ok(())
}
