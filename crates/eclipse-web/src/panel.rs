//! The side panel: the year's eclipses, the chosen one's circumstances, the map's legend,
//! and what a clicked place sees.

use eclipse_core::{
    Eclipse, LocalSolar, LunarEclipse, SolarEclipse,
    geodesy::LatLon,
    time::{CalendarTime, delta_t_at, ut1},
};
use eframe::egui::{self, Color32, RichText, Stroke};
use navigate_celestial::DeltaTBasis;

use crate::{
    palette,
    selection::{Local, Selected},
};

/// The years Navigate's Sun and Moon cover.
pub(crate) const YEARS: std::ops::RangeInclusive<i32> = 1800..=2199;

/// What the viewer asked for in the panel.
pub(crate) enum Action {
    /// Show this year's eclipses.
    Year(i32),
    /// Show the eclipse at this index of the list.
    Select(usize),
}

/// The year chooser and the year's eclipses.
pub(crate) fn year_list(
    ui: &mut egui::Ui,
    year: i32,
    list: &[Eclipse],
    selected: Option<usize>,
) -> Option<Action> {
    let mut action = None;
    ui.horizontal(|ui| {
        if ui.button("◀").clicked() && year > *YEARS.start() {
            action = Some(Action::Year(year - 1));
        }
        let mut chosen = year;
        let drag = ui.add(egui::DragValue::new(&mut chosen).range(YEARS).speed(0.2));
        if drag.changed() {
            action = Some(Action::Year(chosen));
        }
        if ui.button("▶").clicked() && year < *YEARS.end() {
            action = Some(Action::Year(year + 1));
        }
    });
    for (index, eclipse) in list.iter().enumerate() {
        let date = CalendarTime::from_jd(eclipse.greatest_tt()).date();
        let label = match eclipse {
            Eclipse::Solar(e) => format!("☀  {date}  {} solar", e.kind.name()),
            Eclipse::Lunar(e) => format!("🌙  {date}  {} lunar", e.kind.name()),
        };
        if ui
            .selectable_label(selected == Some(index), label)
            .clicked()
        {
            action = Some(Action::Select(index));
        }
    }
    if list.is_empty() {
        ui.label("No eclipses this year.");
    }
    action
}

/// The circumstances of the chosen eclipse, as the Canon's map heads list them.
pub(crate) fn details(ui: &mut egui::Ui, selected: &Selected) {
    match selected {
        Selected::Solar { eclipse, .. } => solar_details(ui, eclipse),
        Selected::Lunar { eclipse, .. } => lunar_details(ui, eclipse),
    }
}

fn heading(ui: &mut egui::Ui, title: String, jd: f64, saros: i64) {
    ui.label(RichText::new(title).heading().strong());
    ui.label(RichText::new(CalendarTime::from_jd(jd).date()).size(18.0));
    ui.label(format!("Saros {saros}"));
}

fn solar_details(ui: &mut egui::Ui, e: &SolarEclipse) {
    let kind = if e.central || e.kind == eclipse_core::SolarKind::Partial {
        e.kind.name().to_owned()
    } else {
        format!("{} (non-central)", e.kind.name())
    };
    heading(ui, format!("{kind} solar eclipse"), e.greatest_tt, e.saros);
    egui::Grid::new("solar details")
        .num_columns(2)
        .show(ui, |ui| {
            row(ui, "Greatest eclipse", times(e.greatest_tt));
            row(ui, "ΔT", delta_t(e.greatest_tt));
            row(ui, "Gamma", format!("{:.4}", e.gamma));
            row(ui, "Magnitude", format!("{:.4}", e.magnitude));
            row(ui, "At", position(e.greatest));
            row(ui, "Sun altitude", format!("{:.0}°", e.sun_altitude));
            if let Some(width) = e.path_width_km {
                row(ui, "Path width", format!("{width:.0} km"));
            }
            if let Some(duration) = e.central_duration_s {
                row(ui, "Duration", minutes_seconds(duration));
            }
        });
}

fn lunar_details(ui: &mut egui::Ui, e: &LunarEclipse) {
    heading(
        ui,
        format!("{} lunar eclipse", e.kind.name()),
        e.greatest_tt,
        e.saros,
    );
    let c = e.contacts;
    let phase = |a: Option<f64>, b: Option<f64>| {
        a.zip(b)
            .map(|(a, b)| format!("{:.0} min", (b - a) * 1440.0))
    };
    egui::Grid::new("lunar details")
        .num_columns(2)
        .show(ui, |ui| {
            row(ui, "Greatest eclipse", times(e.greatest_tt));
            row(ui, "ΔT", delta_t(e.greatest_tt));
            row(ui, "Gamma", format!("{:.4}", e.gamma));
            row(ui, "Umbral magnitude", format!("{:.4}", e.umbral_magnitude));
            row(
                ui,
                "Penumbral magnitude",
                format!("{:.4}", e.penumbral_magnitude),
            );
            for (name, value) in [
                ("Penumbral phase", phase(c.p1, c.p4)),
                ("Partial phase", phase(c.u1, c.u4)),
                ("Totality", phase(c.u2, c.u3)),
            ] {
                if let Some(value) = value {
                    row(ui, name, value);
                }
            }
            for (name, tt) in [
                ("P1", c.p1),
                ("U1", c.u1),
                ("U2", c.u2),
                ("U3", c.u3),
                ("U4", c.u4),
                ("P4", c.p4),
            ] {
                if let Some(tt) = tt {
                    row(ui, name, format!("{} UT", clock(ut1(tt))));
                }
            }
        });
}

/// The map's legend for the chosen eclipse.
pub(crate) fn legend(ui: &mut egui::Ui, selected: &Selected) {
    let entries: &[(&str, Color32, bool)] = match selected {
        Selected::Solar { eclipse, .. } if eclipse.kind == eclipse_core::SolarKind::Partial => &[
            ("Penumbral limits", palette::PENUMBRA, false),
            (
                "Begins or ends at sunrise or sunset",
                palette::PENUMBRA,
                false,
            ),
            (
                "Maximum eclipse at sunrise or sunset",
                palette::MAX_RISE_SET,
                false,
            ),
            ("Magnitude 0.5 at maximum", palette::PENUMBRA, true),
        ],
        Selected::Solar { .. } => &[
            ("Penumbral limits", palette::PENUMBRA, false),
            (
                "Begins or ends at sunrise or sunset",
                palette::PENUMBRA,
                false,
            ),
            (
                "Maximum eclipse at sunrise or sunset",
                palette::MAX_RISE_SET,
                false,
            ),
            ("Magnitude 0.5 at maximum", palette::PENUMBRA, true),
            ("Path of totality or annularity", palette::UMBRA, false),
            ("Central line", palette::UMBRA, true),
        ],
        Selected::Lunar { .. } => &[(
            "Moonrise or moonset at the time shown",
            palette::MAX_RISE_SET,
            false,
        )],
    };
    for (label, color, dashed) in entries {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 12.0), egui::Sense::hover());
            let y = rect.center().y;
            let stroke = Stroke::new(2.5, *color);
            let ends = [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)];
            if *dashed {
                ui.painter()
                    .extend(egui::Shape::dashed_line(&ends, stroke, 4.0, 3.0));
            } else {
                ui.painter().line_segment(ends, stroke);
            }
            ui.label(*label);
        });
    }
    match selected {
        Selected::Solar { .. } => {
            ui.label("✱ greatest eclipse   × Sun overhead at greatest eclipse");
            ui.label("Grey: night, and the shadow, at the time shown");
        }
        Selected::Lunar { .. } => {
            ui.label("White: the whole eclipse is seen; darker: less of it; grey: none of it.");
            ui.label("× Moon overhead at the time shown");
        }
    }
}

/// What the clicked place sees.
pub(crate) fn local(ui: &mut egui::Ui, local: &Local) {
    match local {
        Local::Solar(at, seen) => {
            ui.label(RichText::new(format!("Seen from {}", position(*at))).strong());
            match seen {
                Some(seen) => solar_local(ui, seen),
                None => {
                    ui.label("No eclipse here.");
                }
            }
        }
        Local::Lunar(at, rows) => {
            ui.label(RichText::new(format!("Seen from {}", position(*at))).strong());
            egui::Grid::new("lunar local")
                .num_columns(3)
                .show(ui, |ui| {
                    for (name, tt, altitude) in rows {
                        ui.label(*name);
                        ui.label(format!("{} UT", clock(ut1(*tt))));
                        ui.label(altitude_text(*altitude, "Moon"));
                        ui.end_row();
                    }
                });
        }
    }
}

fn solar_local(ui: &mut egui::Ui, seen: &LocalSolar) {
    egui::Grid::new("solar local")
        .num_columns(3)
        .show(ui, |ui| {
            for (name, contact) in [
                ("Begins", seen.c1),
                ("Central begins", seen.c2),
                ("Maximum", Some(seen.max)),
                ("Central ends", seen.c3),
                ("Ends", seen.c4),
            ] {
                if let Some(contact) = contact {
                    ui.label(name);
                    ui.label(format!("{} UT", clock(ut1(contact.tt))));
                    ui.label(altitude_text(contact.sun_altitude, "Sun"));
                    ui.end_row();
                }
            }
        });
    ui.label(format!(
        "Magnitude {:.3}, {:.1}% of the Sun covered",
        seen.magnitude,
        100.0 * seen.obscuration
    ));
    if let Some(duration) = seen.central_duration_s {
        ui.label(format!(
            "Totality or annularity lasts {}",
            minutes_seconds(duration)
        ));
    }
}

fn altitude_text(degrees: f64, body: &str) -> String {
    if degrees < 0.0 {
        format!("{body} below horizon")
    } else {
        format!("{body} {degrees:.0}° up")
    }
}

fn row(ui: &mut egui::Ui, name: &str, value: String) {
    ui.label(name);
    ui.label(value);
    ui.end_row();
}

/// `HH:MM:SS TD = HH:MM:SS UT`.
pub(crate) fn times(tt: f64) -> String {
    format!("{} TD = {} UT", clock(tt), clock(ut1(tt)))
}

pub(crate) fn clock(jd: f64) -> String {
    CalendarTime::from_jd(jd).time()
}

fn delta_t(tt: f64) -> String {
    let dt = delta_t_at(tt);
    let basis = match dt.basis {
        DeltaTBasis::Observed => "observed",
        DeltaTBasis::Predicted => "IERS prediction",
        DeltaTBasis::Extrapolated => "extrapolated",
    };
    format!(
        "{:.1} s ± {:.1} ({basis})",
        dt.seconds, dt.uncertainty_seconds
    )
}

pub(crate) fn position(at: LatLon) -> String {
    let ns = if at.lat >= 0.0 { 'N' } else { 'S' };
    let ew = if at.lon >= 0.0 { 'E' } else { 'W' };
    format!("{:.1}° {ns}, {:.1}° {ew}", at.lat.abs(), at.lon.abs())
}

fn minutes_seconds(seconds: f64) -> String {
    let s = seconds.round() as i64;
    format!("{}m{:02}s", s / 60, s % 60)
}
