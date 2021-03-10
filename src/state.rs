use crate::vertex::Vertex;
use rand::Rng;
use std::collections::HashMap;
use std::iter;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,

    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,

    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    local_spawner: futures::executor::LocalSpawner,

    vertex_buffer: wgpu::Buffer,

    window_size: winit::dpi::PhysicalSize<u32>,
    shader_compiler: shaderc::Compiler,
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

// new
impl State {
    pub fn new(window: &winit::window::Window) -> Self {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
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
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        Self {
            surface,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            render_pipeline,
            staging_belt: wgpu::util::StagingBelt::new(1024),
            local_pool,
            local_spawner,
            vertex_buffer,
            window_size,
            shader_compiler,
        }
    }
}

// render
impl State {
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        // Encodes a series of GPU operations.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // triangle
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
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

            use wgpu::util::DeviceExt;
            self.vertex_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        // contents: bytemuck::cast_slice(VERTICES),
                        contents: bytemuck::cast_slice(vertices),
                        usage: wgpu::BufferUsage::VERTEX,
                    });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..VERTICES.len() as u32, 0..1);
        }

        // glyphs
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
                    &frame.view,
                    self.window_size.width,
                    self.window_size.height,
                )
                .expect("Draw queued");

            self.staging_belt.finish();
        }

        self.queue.submit(iter::once(encoder.finish()));

        // futures::executor::block_on(self.staging_belt.recall());
        use futures::task::SpawnExt;
        self.local_spawner
            .spawn(self.staging_belt.recall())
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();

        Ok(())
    }
}

// other
impl State {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.window_size = new_size;
        self.swap_chain_descriptor.width = new_size.width;
        self.swap_chain_descriptor.height = new_size.height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}
}

// accessors
impl State {
    pub fn window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window_size
    }
}
