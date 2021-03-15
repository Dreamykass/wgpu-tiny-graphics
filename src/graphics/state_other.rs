use crate::graphics::state::GraphicsState;

// other
impl GraphicsState {
    pub fn resize(&mut self) {
        // self.window_size = new_size;
        let window_size = self.window.inner_size();

        self.swap_chain_descriptor.width = window_size.width;
        self.swap_chain_descriptor.height = window_size.height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    // #[allow(unused_variables)]
    // pub fn input(&mut self, event: &winit::event::Event<()>) -> bool {
    //     self.imgui_platform.handle_event(
    //         self.imgui_context.borrow_mut().io_mut(),
    //         &self.window,
    //         &event,
    //     );
    //     false
    // }
}

// accessors
impl GraphicsState {
    pub fn window_inner_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }
}
