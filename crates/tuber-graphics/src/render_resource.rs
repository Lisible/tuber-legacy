use wgpu::TextureView as WGPUTextureView;

#[derive(Default)]
pub struct RenderResourceStore {
    current_surface_texture_view: Option<WGPUTextureView>,
}

impl RenderResourceStore {
    pub fn store_current_surface_texture_view(
        &mut self,
        current_surface_texture_view: WGPUTextureView,
    ) -> RenderResourceHandle {
        self.current_surface_texture_view = Some(current_surface_texture_view);
        RenderResourceHandle {
            kind: RenderResourceKind::CurrentSurfaceTextureView,
        }
    }

    pub fn texture_view(&self, handle: RenderResourceHandle) -> Option<&WGPUTextureView> {
        return match handle.kind() {
            RenderResourceKind::CurrentSurfaceTextureView => {
                self.current_surface_texture_view.as_ref()
            }
        };
    }
}

pub type PassHandle = usize;

#[derive(Copy, Clone, Debug)]
pub enum RenderResourceSource {
    RenderResource(RenderResourceHandle),
    PassOutput(PassHandle, usize),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RenderResourceHandle {
    kind: RenderResourceKind,
}

impl RenderResourceHandle {
    pub fn kind(&self) -> RenderResourceKind {
        self.kind
    }

    pub fn is_a_texture_view(&self) -> bool {
        self.kind == RenderResourceKind::CurrentSurfaceTextureView
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RenderResourceKind {
    CurrentSurfaceTextureView,
}
