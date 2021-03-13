use crate::graphics_state::state::GraphicsState;

impl<'im> GraphicsState<'im> {
    pub fn render_pass_imgui(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::SwapChainFrame,
    ) {
        let absolute_frame_n = self.frame_counter.absolute_frame_count();
        let last_frame_time = self.frame_counter.last_frame_time();
        let average_frame_time = self.frame_counter.average_frame_time();
        let last_fps = self.frame_counter.last_fps();
        let average_fps = self.frame_counter.average_fps();
        let past_n_fps = self.frame_counter.past_n_fps();

        let frame_metrics = imgui::im_str!(
            "Frame n: {}\nFrame time: {}ms\nAverage frame time: {}\nFPS: {}\nAverage FPS: {}",
            absolute_frame_n,
            last_frame_time.as_secs_f32() * 1000f32,
            average_frame_time,
            last_fps,
            average_fps,
        );

        let mut imgui_context = self.imgui_context.borrow_mut();

        imgui_context.io_mut().update_delta_time(last_frame_time);

        self.imgui_platform
            .prepare_frame(imgui_context.io_mut(), &self.window)
            .expect("Failed to prepare frame");
        let ui = imgui_context.frame();

        // windows
        {
            let window = imgui::Window::new(imgui::im_str!("Hello world"));
            window
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(imgui::im_str!("Hello world!"));
                    ui.text(imgui::im_str!("This...is...imgui-rs on WGPU!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(imgui::im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0],
                        mouse_pos[1]
                    ));
                });

            let window = imgui::Window::new(imgui::im_str!("frame metrics"))
                // .size([800.0, 200.0], imgui::Condition::FirstUseEver)
                .always_auto_resize(true)
                .begin(&ui)
                .unwrap();
            let plot = imgui::PlotLines::new(&ui, &frame_metrics, past_n_fps.as_slice());
            plot.scale_min(0.0)
                .scale_max(60.0)
                .graph_size([600.0, 100.0])
                .build();
            window.end(&ui);

            let window = imgui::Window::new(imgui::im_str!("Hello too"));
            window
                .size([400.0, 200.0], imgui::Condition::FirstUseEver)
                .position([400.0, 200.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(frame_metrics);
                });

            ui.show_demo_window(&mut self.demo_open);
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.imgui_renderer
            .render(ui.render(), &self.queue, &self.device, &mut render_pass)
            .expect("Rendering failed");
    }
}
