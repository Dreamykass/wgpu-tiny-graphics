use crate::graphics::GraphicsState;

pub struct CurrentFrame<'a> {
    pub(super) graphics_state: &'a mut GraphicsState,
    pub(super) frame: wgpu::SwapChainFrame,
    pub(super) encoder: wgpu::CommandEncoder,
}

impl CurrentFrame<'_> {
    pub fn finish_and_present(self) {
        self.graphics_state.staging_belt.finish();

        self.graphics_state
            .queue
            .submit(std::iter::once(self.encoder.finish()));

        use futures::task::SpawnExt;
        self.graphics_state
            .local_spawner
            .spawn(self.graphics_state.staging_belt.recall())
            .expect("Recall staging belt");

        self.graphics_state.local_pool.run_until_stalled();

        self.graphics_state.frame_counter.frame_presented();
    }
}
