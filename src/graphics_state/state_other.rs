use crate::graphics_state::state::GraphicsState;

// other
impl GraphicsState<'_> {
    pub fn resize(&mut self) {
        // self.window_size = new_size;
        self.window_size = self.window.inner_size();

        self.swap_chain_descriptor.width = self.window_size.width;
        self.swap_chain_descriptor.height = self.window_size.height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &winit::event::Event<()>) -> bool {
        self.imgui_platform.handle_event(
            self.imgui_context.borrow_mut().io_mut(),
            &self.window,
            &event,
        );
        false
    }

    pub fn update(&mut self) {}
}

// accessors
impl GraphicsState<'_> {
    pub fn window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window_size
    }
}
