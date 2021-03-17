use crate::graphics::{CurrentFrame, GraphicsState};
use std::cell::RefCell;

pub struct RendererImgui {
    pub imgui_context: &'static RefCell<imgui::Context>,
    pub imgui_platform: &'static RefCell<imgui_winit_support::WinitPlatform>,
    pub(super) imgui_renderer: imgui_wgpu::Renderer,
    #[allow(dead_code)]
    pub(super) imgui_demo_open: bool,
}

// new
impl RendererImgui {
    pub fn new(
        graphics_state: &GraphicsState,
        imgui_context: &'static RefCell<imgui::Context>,
        imgui_platform: &'static RefCell<imgui_winit_support::WinitPlatform>,
    ) -> Self {
        let imgui_renderer = {
            let mut imgui = imgui_context.borrow_mut();
            let hidpi_factor = graphics_state.window.scale_factor();
            imgui.set_ini_filename(None);

            let font_size = (13.0 * hidpi_factor) as f32;
            imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

            imgui
                .fonts()
                .add_font(&[imgui::FontSource::DefaultFontData {
                    config: Some(imgui::FontConfig {
                        oversample_h: 1,
                        pixel_snap_h: true,
                        size_pixels: font_size,
                        ..Default::default()
                    }),
                }]);

            let renderer_config = imgui_wgpu::RendererConfig {
                texture_format: graphics_state.swap_chain_descriptor.format,
                ..Default::default()
            };

            let renderer = imgui_wgpu::Renderer::new(
                &mut imgui,
                &graphics_state.device,
                &graphics_state.queue,
                renderer_config,
            );

            renderer
        };

        RendererImgui {
            imgui_context,
            imgui_platform,
            imgui_renderer,
            imgui_demo_open: false,
        }
    }
}

// draw
impl RendererImgui {
    pub fn draw(&mut self, current_frame: &mut CurrentFrame) {
        let frame_counter = &current_frame.graphics_state.frame_counter;
        let absolute_frame_n = frame_counter.absolute_frame_count();
        let last_frame_time = frame_counter.last_frame_time();
        let average_frame_time = frame_counter.average_frame_time();
        let last_fps = frame_counter.last_fps();
        let average_fps = frame_counter.average_fps();
        let past_n_fps = frame_counter.past_n_fps();

        let frame_metrics = imgui::im_str!(
            "-----------------------------\n\
            Frame n: {}\nFrame time: {:.2}ms\nAverage frame time: {:.2}ms\nFPS: {:.2}\nAverage FPS: {:.2}",
            absolute_frame_n,
            last_frame_time.as_secs_f32() * 1000f32,
            average_frame_time,
            last_fps,
            average_fps,
        );

        let mut imgui_context = self.imgui_context.borrow_mut();
        let imgui_platform = self.imgui_platform.borrow_mut();

        imgui_context.io_mut().update_delta_time(last_frame_time);

        imgui_platform
            .prepare_frame(imgui_context.io_mut(), &current_frame.graphics_state.window)
            .expect("Failed to prepare frame");
        let ui = imgui_context.frame();

        {
            // let window = imgui::Window::new(imgui::im_str!("Hello world"));
            // window
            //     .size([300.0, 100.0], imgui::Condition::FirstUseEver)
            //     .build(&ui, || {
            //         ui.text(imgui::im_str!("Hello world!"));
            //         ui.text(imgui::im_str!("This...is...imgui-rs on WGPU!"));
            //         ui.separator();
            //         let mouse_pos = ui.io().mouse_pos;
            //         ui.text(imgui::im_str!(
            //             "Mouse Position: ({:.1},{:.1})",
            //             mouse_pos[0],
            //             mouse_pos[1]
            //         ));
            //     });

            let window = imgui::Window::new(imgui::im_str!("frame metrics"))
                // .size([800.0, 200.0], imgui::Condition::FirstUseEver)
                .always_auto_resize(true)
                .begin(&ui);
            if let Some(window) = window {
                let plot = imgui::PlotLines::new(&ui, &frame_metrics, past_n_fps.as_slice());
                plot.scale_min(0.0)
                    .scale_max(60.0)
                    .graph_size([600.0, 100.0])
                    .build();
                window.end(&ui);
            }

            // let window = imgui::Window::new(imgui::im_str!("Hello too"));
            // window
            //     .size([400.0, 200.0], imgui::Condition::FirstUseEver)
            //     .position([400.0, 200.0], imgui::Condition::FirstUseEver)
            //     .build(&ui, || {
            //         ui.text(frame_metrics);
            //     });

            // ui.show_demo_window(&mut self.imgui_demo_open);
        }

        let mut render_pass =
            current_frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &current_frame.frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

        self.imgui_renderer
            .render(
                ui.render(),
                &current_frame.graphics_state.queue,
                &current_frame.graphics_state.device,
                &mut render_pass,
            )
            .expect("Rendering failed");
    }
}
