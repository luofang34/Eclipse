//! Pairs the shared map texture with the camera it was drawn with.

use eframe::egui::TextureId;
use egui_wgpu::RenderState;
use maplibre::render::view_state::ViewState;

struct Displayed {
    texture: wgpu::Texture,
    id: TextureId,
    view: ViewState,
}

pub(super) struct Frames {
    host: RenderState,
    displayed: Option<Displayed>,
}

impl Frames {
    pub(super) fn new(host: RenderState) -> Self {
        Self {
            host,
            displayed: None,
        }
    }

    pub(super) fn texture(&self) -> Option<TextureId> {
        self.displayed.as_ref().map(|frame| frame.id)
    }

    pub(super) fn view(&self) -> Option<&ViewState> {
        self.displayed.as_ref().map(|frame| &frame.view)
    }

    pub(super) fn present(&mut self, texture: &wgpu::Texture, view: &ViewState) {
        let texture_view = || texture.create_view(&Default::default());
        if let Some(displayed) = &mut self.displayed {
            if displayed.texture != *texture {
                // A resize replaces the attachment; ordinary frames reuse its binding.
                self.host
                    .renderer
                    .write()
                    .update_egui_texture_from_wgpu_texture(
                        &self.host.device,
                        &texture_view(),
                        wgpu::FilterMode::Linear,
                        displayed.id,
                    );
                displayed.texture = texture.clone();
            }
            displayed.view = view.clone();
        } else {
            let id = self.host.renderer.write().register_native_texture(
                &self.host.device,
                &texture_view(),
                wgpu::FilterMode::Linear,
            );
            self.displayed = Some(Displayed {
                texture: texture.clone(),
                id,
                view: view.clone(),
            });
        }
    }
}

impl Drop for Frames {
    fn drop(&mut self) {
        if let Some(displayed) = self.displayed.take() {
            self.host.renderer.write().free_texture(&displayed.id);
        }
    }
}
