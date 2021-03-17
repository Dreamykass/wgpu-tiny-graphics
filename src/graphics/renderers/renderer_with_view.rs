// #![allow(dead_code)]

use crate::graphics::{CurrentFrame, GraphicsState, SfView};
use crate::vertex::Vertex;
use cgmath::Transform;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    m: [[f32; 4]; 4],
}

pub struct RendererWithView {
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl RendererWithView {
    pub(crate) fn new(graphics_state: &mut GraphicsState) -> Self {
        let uniforms = Uniforms { m: [[0.0; 4]; 4] };

        let uniform_buffer = {
            graphics_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[uniforms]),
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                })
        };

        let uniform_bind_group_layout = {
            graphics_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("uniform_bind_group_layout"),
                })
        };

        let uniform_bind_group = {
            graphics_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    }],
                    label: Some("uniform_bind_group"),
                })
        };

        let pipeline = {
            let (vs_module, fs_module) = {
                let vs_module = crate::shader_compilation::vertex_module(
                    &mut graphics_state.shader_compiler,
                    &graphics_state.device,
                    include_str!("../../shader_vert_with_view.glsl"),
                    "shader.vert",
                    "Vertex Shader",
                )
                .unwrap();

                let fs_module = crate::shader_compilation::fragment_module(
                    &mut graphics_state.shader_compiler,
                    &graphics_state.device,
                    include_str!("../../shader_frag.glsl"),
                    "shader.frag",
                    "Fragment Shader",
                )
                .unwrap();

                (vs_module, fs_module)
            };

            let render_pipeline_layout =
                graphics_state
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[
                            &uniform_bind_group_layout, //     <-- <-- <--
                        ],
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

        use wgpu::util::DeviceExt;

        let buffer = {
            graphics_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &[0; 320],
                    usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                })
        };

        Self {
            pipeline,
            buffer,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    // fn prepare(
    //     graphics_state: &GraphicsState,
    //     frag_name: String,
    //     vert_name: String,
    // ) -> RendererSimpleTriangle {
    //     unimplemented!();
    //     // Renderer {}
    // }

    pub fn draw(&mut self, current_frame: &mut CurrentFrame, view: &SfView) {
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
                    position: [100.0, 100.0, 0.0],
                    color: [0.0, 0.1, 1.0],
                },
                Vertex {
                    position: [900.0, 900.0, 0.0],
                    color: [0.0, 0.2, 1.0],
                },
                Vertex {
                    position: [900.0, 100.0, 0.0],
                    color: [0.0, 0.3, 1.0],
                },
            ]
        };

        current_frame.graphics_state.queue.write_buffer(
            &self.buffer,
            wgpu::BufferAddress::from(0u32),
            bytemuck::cast_slice(vertices),
        );

        // let m: cgmath::Matrix4<f32> = view.get_matrix3().into();
        // let m: cgmath::Matrix4<f32> = m * crate::graphics::sf_view::OPENGL_TO_WGPU_MATRIX4;
        let m = view.get_matrix4();
        let m = m * crate::graphics::sf_view::OPENGL_TO_WGPU_MATRIX4;

        dbg!("oof");
        vertices.iter().for_each(|&v| {
            dbg!(v);
            let p = m.transform_point(cgmath::Point3::new(
                v.position[0],
                v.position[1],
                v.position[2],
            ));
            dbg!(p);
        });

        let u = Uniforms { m: m.into() };

        current_frame.graphics_state.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[u]),
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }
}
