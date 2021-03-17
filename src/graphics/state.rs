use crate::frame_counter::FrameCounter;
use std::cell::RefCell;

pub struct GraphicsState {
    pub window: winit::window::Window,
    pub(super) surface: wgpu::Surface,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,

    pub(super) swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub(super) swap_chain: wgpu::SwapChain,

    pub(super) staging_belt: wgpu::util::StagingBelt,
    pub(super) local_pool: futures::executor::LocalPool,
    pub(super) local_spawner: futures::executor::LocalSpawner,

    #[allow(dead_code)]
    pub(super) shader_compiler: shaderc::Compiler,
    #[allow(dead_code)]
    pub(super) imgui_context: &'static RefCell<imgui::Context>,

    pub(super) frame_counter: FrameCounter,
}
