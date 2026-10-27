//! Choosing a place, by its coordinates or from well-known cities; the eclipses seen from it;
//! and flying down from the globe to stand there.

use eclipse_core::{geodesy::LatLon, time::CalendarTime};
use eframe::egui::{self, RichText};

use super::EclipseApp;
use crate::{
    finder::Finder,
    selection::{Local, Selected},
    sky::{Body, SkyView},
};

/// Places to start from.
const CITIES: [(&str, f64, f64); 14] = [
    ("Reykjavík", 64.147, -21.942),
    ("Madrid", 40.417, -3.704),
    ("Paris", 48.857, 2.352),
    ("London", 51.507, -0.128),
    ("Cairo", 30.044, 31.236),
    ("Beijing", 39.904, 116.407),
    ("Tokyo", 35.676, 139.650),
    ("Sydney", -33.869, 151.209),
    ("Cape Town", -33.925, 18.424),
    ("Mexico City", 19.433, -99.133),
    ("Dallas", 32.777, -96.797),
    ("New York", 40.713, -74.006),
    ("Santiago", -33.449, -70.669),
    ("Buenos Aires", -34.604, -58.382),
];

impl EclipseApp {
    /// The place chooser, the flights down, and the eclipses seen from the place.
    pub(super) fn places_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("A place").strong());
        let mut chosen = None;
        ui.horizontal(|ui| {
            let edit = ui.add(
                egui::TextEdit::singleline(&mut self.place_text)
                    .hint_text("lat, lon  e.g. 40.71, -74.01")
                    .desired_width(170.0),
            );
            let entered = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Go").clicked() || entered {
                chosen = parse(&self.place_text);
                if chosen.is_none() {
                    self.error = Some("Give a place as latitude, longitude in degrees".into());
                }
            }
            egui::ComboBox::from_id_salt("cities")
                .selected_text("Cities")
                .show_ui(ui, |ui| {
                    for (name, lat, lon) in CITIES {
                        if ui.selectable_label(false, name).clicked() {
                            self.place_text = format!("{lat:.3}, {lon:.3}");
                            chosen = Some(LatLon { lat, lon });
                        }
                    }
                });
        });
        if let Some(at) = chosen {
            self.choose_place(at);
        }
        self.fly_buttons(ui);
        self.finder_list(ui);
    }

    fn fly_buttons(&mut self, ui: &mut egui::Ui) {
        let Some(selected) = &self.selected else {
            return;
        };
        let typical = match selected {
            Selected::Solar { eclipse, .. } => Some((
                "Stand at greatest eclipse",
                eclipse.greatest,
                eclipse.greatest_tt,
            )),
            Selected::Lunar { eclipse, .. } => eclipse
                .sublunar(eclipse.greatest_tt)
                .ok()
                .map(|below| ("Stand under the Moon", below, eclipse.greatest_tt)),
        };
        let here = self.local.as_ref().map(|local| {
            let best = match local {
                Local::Solar(_, Some(seen)) => seen.max.tt,
                _ => selected.greatest_tt(),
            };
            ("See the sky from this place", local.at(), best)
        });
        let mut fly = None;
        ui.horizontal_wrapped(|ui| {
            for (label, site, at_tt) in typical.into_iter().chain(here) {
                if ui.button(label).clicked() {
                    fly = Some((site, at_tt));
                }
            }
            if let Some(at) = self.local.as_ref().map(Local::at)
                && ui.button("Find eclipses seen here").clicked()
            {
                self.finder = Some(Finder::new(at, self.year));
            }
        });
        if let Some((site, at_tt)) = fly {
            self.fly_to(site, at_tt);
        }
    }

    fn finder_list(&mut self, ui: &mut egui::Ui) {
        let Some(finder) = &self.finder else { return };
        ui.label(
            RichText::new(format!("Seen from {}", crate::panel::position(finder.site))).strong(),
        );
        let mut open = None;
        for (i, sighting) in finder.found.iter().enumerate() {
            let date = CalendarTime::from_jd(sighting.eclipse.greatest_tt()).date();
            if ui.link(format!("{date}  {}", sighting.what)).clicked() {
                open = Some(i);
            }
        }
        if !finder.done() {
            ui.label(format!("Searching {}…", finder.next_year));
        } else if finder.found.is_empty() {
            ui.label("None seen from here in these years.");
        }
        if let Some(i) = open {
            self.open_sighting(i);
        }
    }

    /// Shows what `at` sees of the eclipse on show.
    pub(super) fn choose_place(&mut self, at: LatLon) {
        let Some(selected) = &self.selected else {
            return;
        };
        match selected.local(at) {
            Ok(local) => self.local = Some(local),
            Err(error) => tracing::error!(%error, "local circumstances"),
        }
    }

    /// Opens a found sighting and flies down to see it.
    fn open_sighting(&mut self, i: usize) {
        let Some((site, greatest, best)) = self.finder.as_ref().and_then(|f| {
            let s = f.found.get(i)?;
            Some((f.site, s.eclipse.greatest_tt(), s.best_tt))
        }) else {
            return;
        };
        self.load_year(CalendarTime::from_jd(greatest).year);
        if let Some(index) = self
            .list
            .iter()
            .position(|e| (e.greatest_tt() - greatest).abs() < 1e-6)
        {
            self.select(index);
            self.choose_place(site);
            self.fly_to(site, best);
        }
    }

    /// Flies down to stand at `site` at `at_tt`, looking at the Sun or the Moon.
    pub(super) fn fly_to(&mut self, site: LatLon, at_tt: f64) {
        let body = match self.selected {
            Some(Selected::Lunar { .. }) => Body::Moon,
            _ => Body::Sun,
        };
        self.now_tt = at_tt;
        self.playing = false;
        self.look = None;
        let from = self.map.as_ref().and_then(|map| map.pose());
        self.sky = Some(SkyView::new(site, body, from));
    }
}

/// Latitude and longitude in degrees from `"lat, lon"`.
fn parse(text: &str) -> Option<LatLon> {
    let mut parts = text.split([',', ' ']).filter(|s| !s.is_empty());
    let lat: f64 = parts.next()?.parse().ok()?;
    let lon: f64 = parts.next()?.parse().ok()?;
    ((-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) && parts.next().is_none())
        .then_some(LatLon { lat, lon })
}
