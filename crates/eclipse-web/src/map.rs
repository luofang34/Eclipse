//! The globe: Natural Earth land and borders drawn by the MapLibre fork into an offscreen
//! texture on the page's GPU, which egui composites. Overlays project through the camera of
//! the frame they are drawn over, so they stay aligned while the globe turns.

use eframe::egui::{Pos2, Rect, TextureId};
use maplibre::{
    coords::LatLon,
    headless::{
        create_headless_renderer_on_host_gpu, environment::HeadlessEnvironment, map::HeadlessMap,
    },
    projection::globe::camera::GlobeCameraState,
    render::{
        Renderer,
        host_gpu::HostGpu,
        projection::globe_camera_for_view,
        settings::RendererSettings,
        view_state::{CameraOptions, NavigationMode},
    },
    style::Style,
    terrain::sightline::{TargetAltitude, Visibility, target_view},
};
use thiserror::Error;

mod frames;
mod gestures;
mod sky;
mod style;

pub use gestures::Gesture;
pub use sky::{GLOBE_FIELD_OF_VIEW, ShadowFrame, SkyFrame};

/// Why the globe could not be created or drawn.
#[derive(Debug, Error)]
pub enum MapError {
    /// The built-in style is not a valid MapLibre style.
    #[error("map style is invalid")]
    Style(#[source] serde_json::Error),
    /// The offscreen renderer could not be created.
    #[error("offscreen renderer could not be created at {width}x{height}")]
    Renderer {
        /// Requested width.
        width: u32,
        /// Requested height.
        height: u32,
        /// Underlying error.
        #[source]
        source: maplibre::headless::HeadlessRendererError,
    },
    /// The map could not be assembled from its style and plugins.
    #[error("map could not be assembled")]
    Assemble(#[source] maplibre::map::MapError),
    /// The free-globe camera could not be set up or moved.
    #[error("globe camera rejected the move")]
    Navigation(#[source] maplibre::render::view_state::NavigationError),
    /// A frame failed to render.
    #[error("map frame failed")]
    Frame(#[source] maplibre::headless::map::HeadlessMapOperationError),
}

/// An offscreen globe at a fixed pixel size.
pub struct MapView {
    map: HeadlessMap,
    frames: frames::Frames,
    gestures: gestures::GestureQueue,
    size: [u32; 2],
    pixel_ratio: f64,
}

/// The camera of a displayed frame, for projecting overlay points onto it.
pub struct Projector {
    camera: GlobeCameraState,
    rect: Rect,
    scale: f32,
}

impl MapView {
    /// A globe of `size` physical pixels on the page's GPU.
    pub fn new(size: [u32; 2], host: &egui_wgpu::RenderState) -> Result<Self, MapError> {
        let style: Style = serde_json::from_value(style::basemap().map_err(MapError::Style)?)
            .map_err(MapError::Style)?;
        let renderer = renderer(size, host)?;
        let gestures = gestures::GestureQueue::default();
        let (kernel, renderer) = renderer;
        let plugins: Vec<Box<dyn maplibre::plugin::Plugin<HeadlessEnvironment>>> = vec![
            Box::new(maplibre::render::RenderPlugin),
            Box::new(gestures.clone()),
            Box::new(maplibre::background::BackgroundPlugin),
            Box::new(maplibre::vector::VectorPlugin::<
                maplibre::vector::DefaultVectorTransferables,
            >::default()),
            Box::new(maplibre::starfield::StarfieldPlugin),
        ];
        let mut map =
            HeadlessMap::new(style, renderer, kernel, plugins).map_err(MapError::Assemble)?;
        map.set_navigation_mode(NavigationMode::FreeGlobe)
            .map_err(MapError::Navigation)?;
        tracing::info!(width = size[0], height = size[1], "globe created");
        Ok(Self {
            map,
            frames: frames::Frames::new(host.clone()),
            gestures,
            size,
            pixel_ratio: 1.0,
        })
    }

    /// The pixel size the globe renders at.
    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    /// Resizes the render target, keeping the camera.
    pub fn resize(&mut self, size: [u32; 2], pixel_ratio: f64) {
        if let Some(physical) = maplibre::window::PhysicalSize::new(size[0], size[1]) {
            self.map.resize(physical);
            self.size = size;
        }
        if pixel_ratio.is_finite() && pixel_ratio > 0.0 {
            self.pixel_ratio = pixel_ratio;
            self.map.set_pixel_ratio(pixel_ratio);
        }
    }

    /// Turns the globe to look straight down on `(lat, lon)` at `zoom`.
    pub fn look_at(&mut self, lat: f64, lon: f64, zoom: f64) -> Result<(), MapError> {
        self.map
            .jump_to(CameraOptions {
                center: Some(LatLon::new(lat, lon)),
                zoom: Some(zoom),
                bearing: Some(cgmath::Deg(0.0)),
                pitch: Some(cgmath::Deg(0.0)),
                ..CameraOptions::default()
            })
            .map_err(MapError::Navigation)
    }

    /// The camera's zoom.
    pub fn zoom(&self) -> f64 {
        self.map.view_state().zoom().value()
    }

    /// Whether the map has work for another frame, such as data still arriving.
    pub fn needs_redraw(&self) -> bool {
        self.map.needs_redraw()
    }

    /// Queues a viewer gesture for the next frame.
    pub fn gesture(&self, gesture: Gesture) {
        self.gestures.push(gesture);
    }

    /// Draws a frame and returns the texture egui shows it from.
    pub fn frame(&mut self) -> Result<Option<TextureId>, MapError> {
        self.map.run_frame().map_err(MapError::Frame)?;
        if let Some(texture) = self.map.head_texture() {
            self.frames.present(texture, self.map.view_state());
        }
        Ok(self.frames.texture())
    }

    /// Projects through the displayed frame's camera into `rect`, the screen area the
    /// texture fills.
    pub fn projector(&self, rect: Rect) -> Option<Projector> {
        self.projector_through(self.frames.view()?, rect)
    }

    /// Projects through the camera as it now stands, before a frame draws it.
    pub fn projector_now(&self, rect: Rect) -> Option<Projector> {
        self.projector_through(self.map.view_state(), rect)
    }

    fn projector_through(
        &self,
        view: &maplibre::render::view_state::ViewState,
        rect: Rect,
    ) -> Option<Projector> {
        let camera = globe_camera_for_view(view).ok()?;
        // The camera works in logical pixels: physical ones over the pixel ratio.
        let logical = self.size[0].max(1) as f32 / self.pixel_ratio as f32;
        let scale = rect.width() / logical;
        Some(Projector {
            camera,
            rect,
            scale,
        })
    }
}

impl Projector {
    /// Where `(lat, lon)` on the sea-level globe shows, or `None` behind it or off the view.
    pub fn project(&self, lat: f64, lon: f64) -> Option<Pos2> {
        self.project_at(lat, lon, 0.0)
    }

    /// Where the point `meters` above sea level at `(lat, lon)` shows, or `None` hidden or
    /// off the view.
    pub fn project_at(&self, lat: f64, lon: f64, meters: f64) -> Option<Pos2> {
        let target = target_view(
            &self.camera,
            None,
            LatLon::new(lat, lon),
            TargetAltitude::Drawn { meters },
        );
        if target.visibility != Visibility::Visible {
            return None;
        }
        let pixel = target.pixel?;
        Some(Pos2::new(
            self.rect.min.x + pixel.x as f32 * self.scale,
            self.rect.min.y + pixel.y as f32 * self.scale,
        ))
    }

    /// The globe's radius on screen in points: how far from `centre` the edge of the
    /// globe shows, found along the great circle toward the equator.
    pub fn globe_radius(&self, centre: (f64, f64)) -> Option<f32> {
        let (lat, lon) = centre;
        let from = eclipse_core::geodesy::LatLon { lat, lon };
        let bearing = if lat > 0.0 { 180.0 } else { 0.0 };
        let at = |angle: f64| {
            let p = crate::overlay::destination(from, bearing, angle);
            self.project(p.lat, p.lon)
        };
        let middle = at(0.0)?;
        let (mut seen, mut hidden) = (0.0, 90.0);
        if at(hidden).is_some() {
            seen = hidden;
        } else {
            for _ in 0..20 {
                let angle = 0.5 * (seen + hidden);
                if at(angle).is_some() {
                    seen = angle
                } else {
                    hidden = angle
                }
            }
        }
        Some(at(seen)?.distance(middle))
    }

    /// The geographic position under a screen point, or `None` off the globe.
    pub fn unproject(&self, at: Pos2) -> Option<(f64, f64)> {
        let pixel = cgmath::Point2::new(
            f64::from((at.x - self.rect.min.x) / self.scale),
            f64::from((at.y - self.rect.min.y) / self.scale),
        );
        let location = self.camera.screen_point_to_location_at(pixel, 0.0)?;
        Some((location.latitude, location.longitude))
    }
}

fn renderer(
    size: [u32; 2],
    host: &egui_wgpu::RenderState,
) -> Result<(maplibre::kernel::Kernel<HeadlessEnvironment>, Renderer), MapError> {
    let gpu = HostGpu {
        instance: host.instance.clone(),
        adapter: host.adapter.clone(),
        device: host.device.clone(),
        queue: host.queue.clone(),
    };
    let settings = RendererSettings {
        texture_format: Some(wgpu::TextureFormat::Rgba8Unorm),
        ..Default::default()
    };
    let [width, height] = size;
    create_headless_renderer_on_host_gpu(width, height, gpu, settings, None).map_err(|source| {
        MapError::Renderer {
            width,
            height,
            source,
        }
    })
}
