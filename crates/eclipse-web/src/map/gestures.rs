//! Viewer gestures, applied to the camera inside the map's frame.

use std::{borrow::Cow, cell::RefCell, rc::Rc};

use cgmath::Point2;
use maplibre::{
    context::MapContext,
    headless::environment::HeadlessEnvironment,
    kernel::Kernel,
    plugin::Plugin,
    render::{RenderStageLabel, graph::RenderGraph},
    schedule::Schedule,
    tcs::{
        system::{System, SystemContainer, SystemResult},
        world::World,
    },
};

/// The globe's widest view: the whole Earth with a margin.
const MIN_ZOOM: f64 = -1.0;
/// The closest view; the basemap holds about 2 km detail.
const MAX_ZOOM: f64 = 7.0;

/// A camera move, in pixels of the map texture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesture {
    /// Turns the globe so the point under `from` comes under `to`.
    Drag {
        /// Where the pointer was.
        from: [f64; 2],
        /// Where it is now.
        to: [f64; 2],
    },
    /// Zooms by `levels` about the pixel `at`.
    Zoom {
        /// The fixed pixel.
        at: [f64; 2],
        /// Zoom levels; positive zooms in.
        levels: f64,
    },
}

/// Gestures waiting for the next frame.
#[derive(Clone, Default)]
pub(super) struct GestureQueue {
    pending: Rc<RefCell<Vec<Gesture>>>,
}

impl GestureQueue {
    pub(super) fn push(&self, gesture: Gesture) {
        self.pending.borrow_mut().push(gesture);
    }
}

impl Plugin<HeadlessEnvironment> for GestureQueue {
    fn build(
        &self,
        schedule: &mut Schedule,
        _: Rc<Kernel<HeadlessEnvironment>>,
        _: &mut World,
        _: &mut RenderGraph,
    ) {
        schedule.add_system_to_stage(
            RenderStageLabel::Extract,
            SystemContainer::new(self.clone()),
        );
    }
}

impl System for GestureQueue {
    fn name(&self) -> Cow<'static, str> {
        "eclipse globe gestures".into()
    }

    fn run(&mut self, context: &mut MapContext) -> SystemResult {
        let view = &mut context.view_state;
        for gesture in self.pending.borrow_mut().drain(..) {
            match gesture {
                Gesture::Drag { from, to } => {
                    view.drag_free_globe(Point2::from(from), Point2::from(to));
                }
                Gesture::Zoom { at, levels } => {
                    let next = (view.zoom().value() + levels).clamp(MIN_ZOOM, MAX_ZOOM);
                    view.zoom_free_globe(Point2::from(at), next - view.zoom().value());
                }
            }
        }
        Ok(())
    }
}
