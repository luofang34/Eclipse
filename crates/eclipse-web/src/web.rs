//! Browser entry: logging, then the App on the page's canvas.

use wasm_bindgen::JsCast;

mod runtime;

/// The canvas `index.html` provides.
const CANVAS_ID: &str = "eclipse";

/// Starts logging and the App; a failure is logged and shown in place of the canvas.
pub(crate) fn start() {
    console_error_panic_hook::set_once();
    use tracing_subscriber::Layer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let console = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_writer(tracing_web::MakeWebConsoleWriter::new())
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
    tracing_subscriber::registry().with(console).init();
    wasm_bindgen_futures::spawn_local(async {
        if let Err(error) = run().await {
            tracing::error!(%error, "eclipse maps could not start");
            show_status(&error);
        }
    });
}

async fn run() -> Result<(), String> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or("the page has no document")?;
    let canvas = document
        .get_element_by_id(CANVAS_ID)
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or("the page has no canvas")?;
    runtime::run(canvas).await
}

/// Takes the loading notice off the page once the App draws.
fn page_started() {
    if let Some(status) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("status"))
    {
        status.remove();
    }
}

fn show_status(message: &str) {
    if let Some(status) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("status"))
    {
        status.set_text_content(Some(message));
    }
}

/// The Julian date now, by the browser's clock; close enough to TT to pick an eclipse.
pub(crate) fn today_jd() -> f64 {
    let seconds = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64());
    2_440_587.5 + seconds / eclipse_core::time::SECONDS_PER_DAY
}

/// The `date` the page's query names, as `YYYY-MM-DD`, or `-YYYY-MM-DD` before year 0.
pub(crate) fn query_date() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|pair| pair.strip_prefix("date="))
        .filter(|date| date.trim_start_matches('-').len() == 10)
        .map(str::to_owned)
}

/// Puts `date` in the page's address, so the view can be shared.
pub(crate) fn show_date(date: &str) {
    let Some(history) = web_sys::window().and_then(|window| window.history().ok()) else {
        return;
    };
    let url = format!("?date={date}");
    if let Err(error) = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url))
    {
        tracing::warn!(?error, "address not updated");
    }
}
