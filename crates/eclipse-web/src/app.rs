//! The page: the panel of eclipses and circumstances, the globe with the chosen eclipse's
//! map, the lunar figure, and the time bar.

use eclipse_core::{Eclipse, eclipses_in_year, geodesy::LatLon, time::CalendarTime};
use eframe::egui::{self, Color32, Sense};
use egui_wgpu::{WgpuConfiguration, WgpuSetupCreateNew};
use maplibre::render::view_state::GlobePose;

use crate::{
    figure,
    finder::Finder,
    globe,
    map::{Gesture, MapError, MapView},
    panel::{self, Action},
    selection::{Local, Selected},
    sky::SkyView,
    web,
};

mod places;
mod sky_mode;

/// Days of eclipse a second of playing shows: an hour in twelve seconds.
const PLAY_RATE: f64 = 5.0 / 1440.0;
/// The globe's radius as a share of the smaller side of its view.
const GLOBE_SHARE: f32 = 0.42;

/// The page's GPU device, created without a display handle as the map renderer needs.
pub(crate) fn gpu_configuration() -> WgpuConfiguration {
    WgpuConfiguration {
        wgpu_setup: WgpuSetupCreateNew::without_display_handle().into(),
        ..Default::default()
    }
}

/// The App's state.
pub(crate) struct EclipseApp {
    host: egui_wgpu::RenderState,
    map: Option<MapView>,
    year: i32,
    list: Vec<Eclipse>,
    index: Option<usize>,
    selected: Option<Selected>,
    now_tt: f64,
    playing: bool,
    local: Option<Local>,
    error: Option<String>,
    /// Where the globe turns to once it can.
    look: Option<LatLon>,
    /// The camera a rebuilt globe takes over from the one the GPU lost.
    restored_pose: Option<GlobePose>,
    /// The view from the ground, while the eye stands there.
    sky: Option<SkyView>,
    /// The search for eclipses seen from a place.
    finder: Option<Finder>,
    /// The place typed in.
    place_text: String,
}

impl EclipseApp {
    /// The App on the page's GPU, showing the eclipse the page's `?date=` names, else the
    /// year's next one.
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let host = cc
            .wgpu_render_state
            .clone()
            .ok_or("the page has no wgpu renderer")?;
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let today = web::today_jd();
        let wanted = web::query_date();
        let year = wanted
            .as_deref()
            .and_then(|date| {
                let year: i32 = date.trim_start_matches('-').get(..4)?.parse().ok()?;
                Some(if date.starts_with('-') { -year } else { year })
            })
            .unwrap_or(CalendarTime::from_jd(today).year)
            .clamp(*panel::YEARS.start(), *panel::YEARS.end());
        let mut app = Self {
            host,
            map: None,
            year,
            list: Vec::new(),
            index: None,
            selected: None,
            now_tt: today,
            playing: false,
            local: None,
            error: None,
            look: None,
            restored_pose: None,
            sky: None,
            finder: None,
            place_text: String::new(),
        };
        app.load_year(year);
        let upcoming = |list: &[Eclipse]| list.iter().position(|e| e.greatest_tt() > today);
        // Without a date, the next eclipse to come, in the following year if this one has
        // no more.
        if wanted.is_none() && upcoming(&app.list).is_none() && year < *panel::YEARS.end() {
            app.load_year(year + 1);
        }
        let index = app
            .list
            .iter()
            .position(|e| Some(CalendarTime::from_jd(e.greatest_tt()).iso_date()) == wanted)
            .or_else(|| upcoming(&app.list))
            .or(if app.list.is_empty() { None } else { Some(0) });
        if let Some(index) = index {
            app.select(index);
        }
        Ok(app)
    }

    fn load_year(&mut self, year: i32) {
        self.year = year;
        self.index = None;
        self.selected = None;
        self.local = None;
        let started = web_time::Instant::now();
        match eclipses_in_year(year) {
            Ok(list) => {
                tracing::info!(
                    year,
                    found = list.len(),
                    ms = started.elapsed().as_millis(),
                    "eclipses found"
                );
                self.list = list;
                self.error = None;
            }
            Err(error) => {
                tracing::error!(%error, year, "eclipses unavailable");
                self.list = Vec::new();
                self.error = Some(error.to_string());
            }
        }
    }

    fn select(&mut self, index: usize) {
        if self.sky.is_some() {
            self.leave_sky();
        }
        let Some(eclipse) = self.list.get(index) else {
            return;
        };
        let started = web_time::Instant::now();
        match Selected::of(eclipse) {
            Ok(selected) => {
                tracing::info!(ms = started.elapsed().as_millis(), "eclipse map computed");
                self.now_tt = selected.greatest_tt();
                self.look = Some(selected.focus());
                web::show_date(&CalendarTime::from_jd(selected.greatest_tt()).iso_date());
                self.selected = Some(selected);
                self.index = Some(index);
                self.local = None;
                self.playing = false;
                self.error = None;
            }
            Err(error) => {
                tracing::error!(%error, "eclipse map unavailable");
                self.error = Some(error.to_string());
            }
        }
    }

    fn side_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(egui::RichText::new("Eclipse Maps").heading().strong());
        ui.label("Solar and lunar eclipses, 2000 BCE to 2999");
        ui.small("Dates before 1582 October 15 are Julian; years are astronomical, 0 = 1 BCE.");
        ui.separator();
        if let Some(action) = panel::year_list(ui, self.year, &self.list, self.index) {
            match action {
                Action::Year(year) => {
                    self.load_year(year);
                    if !self.list.is_empty() {
                        self.select(0);
                    }
                }
                Action::Select(index) => self.select(index),
            }
        }
        if let Some(error) = &self.error {
            ui.colored_label(Color32::DARK_RED, error);
        }
        if let Some(selected) = &self.selected {
            ui.separator();
            panel::details(ui, selected);
            ui.separator();
            panel::legend(ui, selected);
        }
        ui.separator();
        if self.sky.is_some() {
            self.sky_panel(ui);
            ui.separator();
        }
        match &self.local {
            Some(local) => panel::local(ui, local),
            None => {
                ui.label(
                    "Click the globe to see the eclipse from that place. \
                     Drag to turn it, scroll to zoom.",
                );
            }
        }
        ui.separator();
        self.places_panel(ui);
        ui.separator();
        ui.small(
            "Sun and Moon from Navigate's analytic ephemeris; ΔT from IERS and USNO data. \
             Shadows and lunar radii as in NASA's Five Millennium Canons (Espenak and Meeus). \
             Coastlines and borders: Natural Earth.",
        );
    }

    fn time_bar(&mut self, ui: &mut egui::Ui) {
        let Some(selected) = &self.selected else {
            return;
        };
        let (start, end) = selected.span();
        let mut export = false;
        ui.horizontal(|ui| {
            let label = if self.playing { "⏸" } else { "▶" };
            if ui.button(label).clicked() {
                if !self.playing && self.now_tt >= end {
                    self.now_tt = start;
                }
                self.playing = !self.playing;
            }
            if ui.button("Greatest").clicked() {
                self.now_tt = selected.greatest_tt();
                self.playing = false;
            }
            export = ui
                .button("Save SVG")
                .on_hover_text("Save this eclipse as an animated SVG")
                .clicked();
            ui.label(panel::times(self.now_tt));
            ui.spacing_mut().slider_width = (ui.available_width() - 20.0).max(100.0);
            let slider = egui::Slider::new(&mut self.now_tt, start..=end).show_value(false);
            if ui.add(slider).dragged() {
                self.playing = false;
            }
        });
        if export && let Err(error) = crate::export::save(selected) {
            tracing::error!(%error, "SVG export");
            self.error = Some(error);
        }
        if self.playing {
            self.now_tt += PLAY_RATE * f64::from(ui.input(|input| input.stable_dt.min(0.1)));
            if self.now_tt >= end {
                self.now_tt = end;
                self.playing = false;
            }
            ui.ctx().request_repaint();
        }
    }

    /// Moves the App onto `host`, the page's new GPU after the old one was lost, keeping the
    /// eclipse, the time, the place and the camera on show.
    pub(crate) fn restore_gpu(
        &mut self,
        cc: &eframe::CreationContext<'_>,
        host: egui_wgpu::RenderState,
    ) {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        self.restored_pose = self.map.take().and_then(|map| map.pose());
        self.host = host;
        if let Some(sky) = &mut self.sky {
            sky.resend_stars();
        }
    }

    fn globe(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let ppp = ui.ctx().pixels_per_point();
        let size = [
            (rect.width() * ppp).round().max(1.0) as u32,
            (rect.height() * ppp).round().max(1.0) as u32,
        ];
        if self.map.is_none() {
            match MapView::new(size, &self.host) {
                Ok(mut map) => {
                    if let Some(pose) = self.restored_pose.take()
                        && let Err(error) = map.set_pose(pose)
                    {
                        tracing::warn!(%error, "globe camera not restored");
                    }
                    self.map = Some(map);
                }
                Err(error) => {
                    tracing::error!(%error, "globe unavailable");
                    ui.label(format!("The globe could not start: {error}"));
                    return;
                }
            }
        }
        if let Some(map) = &mut self.map
            && map.size() != size
        {
            map.resize(size, f64::from(ppp));
        }
        if self.sky.is_some() {
            let response = ui.allocate_rect(rect, Sense::click_and_drag());
            if self.sky_globe(ui, rect, &response) {
                ui.ctx().request_repaint();
            }
            return;
        }
        let Some(map) = &mut self.map else { return };
        if let Some(at) = self.look.take() {
            let wanted = GLOBE_SHARE * rect.width().min(rect.height());
            if let Err(error) = fit(map, at, wanted, rect) {
                tracing::error!(%error, "globe camera");
            }
        }
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let moved = gestures(ui, &response, map, rect);
        let mut waiting = false;
        match map.frame() {
            Ok(Some(texture)) => {
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(texture, rect, uv, Color32::WHITE);
            }
            Ok(None) => waiting = true,
            Err(error) => tracing::error!(%error, "globe frame"),
        }
        let painter = ui.painter_at(rect);
        if let (Some(projector), Some(selected)) = (map.projector(rect), &self.selected) {
            globe::draw(&painter, &projector, selected, self.now_tt);
            if response.clicked()
                && let Some(at) = response.interact_pointer_pos()
                && let Some((lat, lon)) = projector.unproject(at)
            {
                match selected.local(LatLon { lat, lon }) {
                    Ok(local) => self.local = Some(local),
                    Err(error) => tracing::error!(%error, "local circumstances"),
                }
            }
        }
        if let Some(Selected::Lunar { eclipse, .. }) = &self.selected {
            let side = (0.36 * rect.width().min(rect.height())).clamp(180.0, 320.0);
            let inset = egui::Rect::from_min_size(
                rect.min + egui::vec2(12.0, 12.0),
                egui::vec2(side, side),
            );
            if let Err(error) = figure::draw(&painter, inset, eclipse, self.now_tt) {
                tracing::error!(%error, "lunar figure");
            }
        }
        if moved || waiting || map.needs_redraw() {
            ui.ctx().request_repaint();
        }
    }
}

/// Turns the globe to `at` and zooms until its radius is `wanted` points. The radius does
/// not grow as 2^zoom, since the camera comes closer as it zooms, so each round takes the
/// slope of log radius over zoom from the last two.
fn fit(map: &mut MapView, at: LatLon, wanted: f32, rect: egui::Rect) -> Result<(), MapError> {
    // From the ground the camera's zoom is far past any that shows the whole globe.
    let mut zoom = map.zoom().clamp(-1.0, 4.0);
    let mut last: Option<(f64, f64)> = None;
    for _ in 0..12 {
        map.look_at(at.lat, at.lon, zoom)?;
        let Some(radius) = map
            .projector_now(rect)
            .and_then(|projector| projector.globe_radius((at.lat, at.lon)))
        else {
            return Ok(());
        };
        let log_radius = f64::from(radius.max(1.0)).log2();
        let error = f64::from(wanted).log2() - log_radius;
        if error.abs() < 0.005 {
            break;
        }
        let slope = last
            .filter(|(z, _)| (zoom - z).abs() > 1e-3)
            .map(|(z, r)| (log_radius - r) / (zoom - z))
            .filter(|slope| (0.5..4.0).contains(slope))
            .unwrap_or(1.0);
        last = Some((zoom, log_radius));
        zoom += error / slope;
    }
    Ok(())
}

/// Passes drags and scrolls on the globe to its camera; true when one moved it.
fn gestures(ui: &egui::Ui, response: &egui::Response, map: &MapView, rect: egui::Rect) -> bool {
    let to_pixels = |p: egui::Pos2| [f64::from(p.x - rect.min.x), f64::from(p.y - rect.min.y)];
    let mut moved = false;
    if response.dragged()
        && let Some(now) = response.interact_pointer_pos()
    {
        map.gesture(Gesture::Drag {
            from: to_pixels(now - response.drag_delta()),
            to: to_pixels(now),
        });
        moved = true;
    }
    let scroll = ui.input(|input| input.smooth_scroll_delta.y);
    if response.hovered()
        && scroll != 0.0
        && let Some(at) = response.hover_pos()
    {
        map.gesture(Gesture::Zoom {
            at: to_pixels(at),
            levels: f64::from(scroll) / 200.0,
        });
        moved = true;
    }
    moved
}

impl eframe::App for EclipseApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(finder) = &mut self.finder
            && !finder.done()
        {
            finder.step();
            ui.ctx().request_repaint();
        }
        egui::Panel::left("panel")
            .resizable(false)
            .exact_size(340.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.side_panel(ui));
            });
        egui::Panel::bottom("time").show(ui, |ui| {
            ui.add_space(4.0);
            self.time_bar(ui);
            ui.add_space(4.0);
        });
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| self.globe(ui));
    }
}
