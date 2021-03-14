use crate::graphics::GraphicsState;

pub struct CurrentFrame<'a> {
    pub(super) graphics_state: &'a mut GraphicsState,
    pub(super) frame: wgpu::SwapChainFrame,
    pub(super) encoder: wgpu::CommandEncoder,
}
