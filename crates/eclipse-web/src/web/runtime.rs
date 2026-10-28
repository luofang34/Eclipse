//! Runs the App on the page's GPU, and on a new one when the browser takes it away, as Safari
//! does with a page left idle, keeping the App's state.

use std::{cell::RefCell, rc::Rc, sync::Mutex};

use eframe::egui;
use futures_channel::oneshot;

use crate::app::EclipseApp;

type SharedApp = Rc<RefCell<Option<EclipseApp>>>;

/// Starts the App on `canvas` and restarts its renderer each time the GPU device is lost.
pub(super) async fn run(canvas: web_sys::HtmlCanvasElement) -> Result<(), String> {
    let runner = eframe::WebRunner::new();
    let app: SharedApp = Rc::default();
    loop {
        let (sender, lost) = oneshot::channel();
        let shared = app.clone();
        runner
            .start(
                canvas.clone(),
                eframe::WebOptions {
                    wgpu_options: crate::app::gpu_configuration(),
                    ..Default::default()
                },
                Box::new(move |cc| {
                    let host = cc
                        .wgpu_render_state
                        .clone()
                        .ok_or("the page has no wgpu renderer")?;
                    watch_device(&host.device, sender);
                    let mut app = shared.borrow_mut();
                    if let Some(app) = app.as_mut() {
                        app.restore_gpu(cc, host);
                        tracing::info!("page GPU restored");
                    } else {
                        *app = Some(EclipseApp::new(cc)?);
                    }
                    drop(app);
                    Ok(Box::new(HostedApp(shared)))
                }),
            )
            .await
            .map_err(|error| format!("{error:?}"))?;
        super::page_started();
        let (reason, message) = lost
            .await
            .map_err(|error| format!("GPU loss notification failed: {error}"))?;
        // Stop the frame loop before it draws with resources the lost device invalidated.
        runner.destroy();
        tracing::warn!(?reason, %message, "page GPU lost; rebuilding the renderer");
    }
}

fn watch_device(device: &wgpu::Device, sender: oneshot::Sender<(wgpu::DeviceLostReason, String)>) {
    let sender = Mutex::new(Some(sender));
    device.set_device_lost_callback(move |reason, message| {
        if let Some(sender) = sender.lock().ok().and_then(|mut sender| sender.take()) {
            sender.send((reason, message)).ok();
        }
    });
}

/// The App as the runner holds it, shared with the loop that restarts the runner.
struct HostedApp(SharedApp);

impl eframe::App for HostedApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if let Some(app) = self.0.borrow_mut().as_mut() {
            app.ui(ui, frame);
        }
    }
}
