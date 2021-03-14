use crate::frame_counter::FrameCounter;
use std::cell::RefCell;

pub struct GraphicsState {
    pub window: winit::window::Window,
    pub(super) surface: wgpu::Surface,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,

    pub(super) swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub(super) swap_chain: wgpu::SwapChain,
    pub(super) render_pipeline: wgpu::RenderPipeline,

    pub(super) staging_belt: wgpu::util::StagingBelt,
    pub(super) local_pool: futures::executor::LocalPool,
    pub(super) local_spawner: futures::executor::LocalSpawner,

    pub(super) imgui_context: &'static RefCell<imgui::Context>,
    pub(super) imgui_renderer: imgui_wgpu::Renderer,
    pub(super) imgui_platform: imgui_winit_support::WinitPlatform,
    pub(super) imgui_demo_open: bool,

    pub(super) vertex_buffer: wgpu::Buffer,

    #[allow(dead_code)]
    pub(super) shader_compiler: shaderc::Compiler,

    pub(super) frame_counter: FrameCounter,
}
