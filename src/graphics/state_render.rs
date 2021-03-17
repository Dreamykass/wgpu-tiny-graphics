use crate::graphics::state::GraphicsState;
use crate::graphics::CurrentFrame;

// new render
impl GraphicsState {
    pub fn begin_current_frame(&mut self) -> Result<CurrentFrame, wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?;
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        Ok(CurrentFrame {
            graphics_state: self,
            frame,
            encoder,
        })
    }
}
