#![allow(dead_code)]

use crate::graphics::GraphicsState;

pub struct RendererSimpleTriangle {
    frag_name: String,
    vert_name: String,
    layout: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
}

impl RendererSimpleTriangle {
    fn prepare(
        graphics_state: &GraphicsState,
        frag_name: String,
        vert_name: String,
    ) -> RendererSimpleTriangle {
        unimplemented!();
        // Renderer {}
    }
}
