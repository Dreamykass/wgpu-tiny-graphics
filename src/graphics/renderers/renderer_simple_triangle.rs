#![allow(dead_code)]

use crate::graphics::{CurrentFrame, GraphicsState};
use crate::vertex::Vertex;
use rand::Rng;

pub struct RendererSimpleTriangle {
    // frag_name: String,
    // vert_name: String,
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
}

impl RendererSimpleTriangle {
    pub(crate) fn new(graphics_state: &mut GraphicsState) -> Self {
        let pipeline = {
            let vs_module = crate::shader_compilation::vertex_module(
                &mut graphics_state.shader_compiler,
                &graphics_state.device,
                include_str!("../../shader.vert"),
                "shader.vert",
                "Vertex Shader",
            )
            .unwrap();

            let fs_module = crate::shader_compilation::fragment_module(
                &mut graphics_state.shader_compiler,
                &graphics_state.device,
                include_str!("../../shader.frag"),
                "shader.frag",
                "Fragment Shader",
            )
            .unwrap();

            let render_pipeline_layout =
                graphics_state
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });

            graphics_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                            format: graphics_state.swap_chain_descriptor.format,
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

        let buffer = {
            use wgpu::util::DeviceExt;
            graphics_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &[0; 640],
                    usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                })
        };

        RendererSimpleTriangle { pipeline, buffer }
    }

    // fn prepare(
    //     graphics_state: &GraphicsState,
    //     frag_name: String,
    //     vert_name: String,
    // ) -> RendererSimpleTriangle {
    //     unimplemented!();
    //     // Renderer {}
    // }

    pub fn draw(&mut self, current_frame: &mut CurrentFrame) {
        let mut render_pass =
            current_frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &current_frame.frame.output.view,
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

        current_frame.graphics_state.queue.write_buffer(
            &self.buffer,
            wgpu::BufferAddress::from(0u32),
            bytemuck::cast_slice(vertices),
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }
}
