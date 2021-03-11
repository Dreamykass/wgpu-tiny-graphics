use crate::frame_counter::FrameCounter;
use crate::vertex::Vertex;
use rand::Rng;
use std::iter;
use std::time::Instant;

pub struct GraphicsState {
    pub window: winit::window::Window,
    window_size: winit::dpi::PhysicalSize<u32>,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,

    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,

    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    local_spawner: futures::executor::LocalSpawner,

    pub imgui_context: imgui::Context,
    imgui_renderer: imgui_wgpu::Renderer,
    imgui_platform: imgui_winit_support::WinitPlatform,
    demo_open: bool,

    vertex_buffer: wgpu::Buffer,

    #[allow(dead_code)]
    shader_compiler: shaderc::Compiler,

    frame_counter: FrameCounter,
}

// new
impl GraphicsState {
    pub fn new(window: winit::window::Window) -> Self {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            }))
            .unwrap();

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        ))
        .unwrap();

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        let mut shader_compiler = shaderc::Compiler::new().unwrap();

        let render_pipeline = {
            let vs_module = crate::shader_compilation::vertex_module(
                &mut shader_compiler,
                &device,
                include_str!("shader.vert"),
                "shader.vert",
                "Vertex Shader",
            )
            .unwrap();

            let fs_module = crate::shader_compilation::fragment_module(
                &mut shader_compiler,
                &device,
                include_str!("shader.frag"),
                "shader.frag",
                "Fragment Shader",
            )
            .unwrap();

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

            // ret from block
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &[Vertex::descriptor()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: swap_chain_descriptor.format,
                        alpha_blend: wgpu::BlendState::REPLACE,
                        color_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::Back,
                    polygon_mode: wgpu::PolygonMode::Fill,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            })
        }; // let render_pipeline =

        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[0; 640],
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let (imgui_context, imgui_renderer, imgui_platform) = {
            let hidpi_factor = window.scale_factor();
            let mut imgui = imgui::Context::create();
            let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
            platform.attach_window(
                imgui.io_mut(),
                &window,
                imgui_winit_support::HiDpiMode::Default,
            );
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
                texture_format: swap_chain_descriptor.format,
                ..Default::default()
            };

            let renderer = imgui_wgpu::Renderer::new(&mut imgui, &device, &queue, renderer_config);

            (imgui, renderer, platform)
        };

        Self {
            window,
            surface,
            device,

            queue,
            swap_chain_descriptor,
            swap_chain,
            render_pipeline,

            staging_belt: wgpu::util::StagingBelt::new(1024),
            local_pool,
            local_spawner,

            imgui_context,
            imgui_renderer,
            imgui_platform,
            demo_open: true,

            vertex_buffer,

            window_size,

            shader_compiler,

            frame_counter: FrameCounter::default(),
        }
    }
}

// render
impl GraphicsState {
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let absolute_frame_n = self.frame_counter.absolute_frame_count();
        let last_frame_time = self.frame_counter.last_frame_time();
        let average_frame_time = self.frame_counter.average_frame_time();
        let last_fps = self.frame_counter.last_fps();
        let average_fps = self.frame_counter.average_fps();

        let frame = self.swap_chain.get_current_frame()?;

        // Encodes a series of GPU operations.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // triangle (1st render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            let vertices: &[Vertex] = {
                &[
                    Vertex {
                        position: [0.0, 0.5, 0.0],
                        color: [
                            rand::thread_rng().gen_range(0.0..1.0),
                            rand::thread_rng().gen_range(0.0..1.0),
                            rand::thread_rng().gen_range(0.0..1.0),
                        ],
                    },
                    Vertex {
                        position: [-0.5, -0.5, 0.0],
                        color: [
                            rand::thread_rng().gen_range(0.0..1.0),
                            rand::thread_rng().gen_range(0.0..1.0),
                            rand::thread_rng().gen_range(0.0..1.0),
                        ],
                    },
                    Vertex {
                        position: [0.5, -0.5, 0.0],
                        color: [
                            rand::thread_rng().gen_range(0.0..1.0),
                            rand::thread_rng().gen_range(0.0..1.0),
                            rand::thread_rng().gen_range(0.0..1.0),
                        ],
                    },
                ]
            };

            self.queue.write_buffer(
                &self.vertex_buffer,
                wgpu::BufferAddress::from(0u32),
                bytemuck::cast_slice(vertices),
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        // glyphs (2nd internal render pass)
        {
            let inconsolata = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
                "Inconsolata-Regular.ttf"
            ))
            .unwrap();

            let mut glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(inconsolata)
                .build(&self.device, self.swap_chain_descriptor.format);

            glyph_brush.queue(wgpu_glyph::Section {
                screen_position: (30.0, 30.0),
                bounds: (
                    self.window_size.width as f32,
                    self.window_size.height as f32,
                ),
                text: vec![wgpu_glyph::Text::new("Hello wgpu_glyph!")
                    .with_color([0.0, 0.0, 0.0, 1.0])
                    .with_scale(40.0)],
                ..wgpu_glyph::Section::default()
            });

            glyph_brush.queue(wgpu_glyph::Section {
                screen_position: (30.0, 90.0),
                bounds: (
                    self.window_size.width as f32,
                    self.window_size.height as f32,
                ),
                text: vec![wgpu_glyph::Text::new(&*format!(
                    "Hello wgpu_glyph! {}",
                    rand::thread_rng().gen_range(0..100)
                ))
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(40.0)],
                ..wgpu_glyph::Section::default()
            });

            glyph_brush
                .draw_queued(
                    &self.device,
                    &mut self.staging_belt,
                    &mut encoder,
                    &frame.output.view,
                    self.window_size.width,
                    self.window_size.height,
                )
                .expect("Draw queued");

            self.staging_belt.finish();
        }

        // imgui (3rd render pass)
        {
            self.imgui_context
                .io_mut()
                .update_delta_time(last_frame_time);

            self.imgui_platform
                .prepare_frame(self.imgui_context.io_mut(), &self.window)
                .expect("Failed to prepare frame");
            let ui = self.imgui_context.frame();

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

                let window = imgui::Window::new(imgui::im_str!("Hello too"));
                window
                    .size([400.0, 200.0], imgui::Condition::FirstUseEver)
                    .position([400.0, 200.0], imgui::Condition::FirstUseEver)
                    .build(&ui, || {
                        ui.text(imgui::im_str!(
                            "Frame n: {}\nFrame time: {}ms\nAverage frame time: {}\nFPS: {}\nAverage FPS: {}",
                            absolute_frame_n,
                            last_frame_time.as_secs_f32() * 1000f32,
                            average_frame_time,
                            last_fps,
                            average_fps,
                        ));
                    });

                ui.show_demo_window(&mut self.demo_open);

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

        self.queue.submit(iter::once(encoder.finish()));

        use futures::task::SpawnExt;
        self.local_spawner
            .spawn(self.staging_belt.recall())
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();

        self.frame_counter.frame_presented();

        Ok(())
    }
}

// other
impl GraphicsState {
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
        self.imgui_platform
            .handle_event(self.imgui_context.io_mut(), &self.window, &event);
        false
    }

    pub fn update(&mut self) {}
}

// accessors
impl GraphicsState {
    pub fn window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window_size
    }
}
