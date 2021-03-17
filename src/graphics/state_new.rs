use crate::frame_counter::FrameCounter;
use crate::graphics::state::GraphicsState;
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

        let shader_compiler = shaderc::Compiler::new().unwrap();

        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        Self {
            window,
            surface,
            device,

            queue,
            swap_chain_descriptor,
            swap_chain,

            staging_belt: wgpu::util::StagingBelt::new(1024),
            local_pool,
            local_spawner,

            shader_compiler,

            imgui_context,
            frame_counter: FrameCounter::default(),
        }
    }
}
