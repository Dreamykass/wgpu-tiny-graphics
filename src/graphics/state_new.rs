use crate::frame_counter::FrameCounter;
use crate::graphics::state::GraphicsState;
use crate::vertex::Vertex;
use std::cell::RefCell;

// new
impl GraphicsState {
    pub fn new(
        window: winit::window::Window,
        imgui_context: &'static RefCell<imgui::Context>,
    ) -> Self {
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
                include_str!("../shader.vert"),
                "shader.vert",
                "Vertex Shader",
            )
            .unwrap();

            let fs_module = crate::shader_compilation::fragment_module(
                &mut shader_compiler,
                &device,
                include_str!("../shader.frag"),
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

        let (imgui_renderer, imgui_platform) = {
            let mut imgui = imgui_context.borrow_mut();
            let hidpi_factor = window.scale_factor();
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

            (renderer, platform)
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
            imgui_demo_open: true,

            vertex_buffer,

            shader_compiler,

            frame_counter: FrameCounter::default(),
        }
    }
}
