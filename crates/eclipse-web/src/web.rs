//! Browser entry: logging, then the App on the page's canvas.

use wasm_bindgen::JsCast;

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
    let options = eframe::WebOptions {
        wgpu_options: crate::app::gpu_configuration(),
        ..Default::default()
    };
    eframe::WebRunner::new()
        .start(
            canvas,
            options,
            Box::new(|cc| Ok(Box::new(crate::app::EclipseApp::new(cc)?))),
        )
        .await
        .map_err(|error| format!("{error:?}"))?;
    if let Some(status) = document.get_element_by_id("status") {
        status.remove();
    }
    Ok(())
}

fn show_status(message: &str) {
    if let Some(status) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("status"))
    {
        status.set_text_content(Some(message));
    }
}
