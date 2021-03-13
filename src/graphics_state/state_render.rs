use crate::graphics_state::state::GraphicsState;
use crate::vertex::Vertex;
use rand::Rng;

// render
impl GraphicsState<'_> {
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?;

        // Encodes a series of GPU operations.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // triangle (1st render pass)
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
        self.render_pass_glyph(&mut encoder, &frame);

        // imgui (3rd internal render pass)
        self.render_pass_imgui(&mut encoder, &frame);

        self.queue.submit(std::iter::once(encoder.finish()));

        use futures::task::SpawnExt;
        self.local_spawner
            .spawn(self.staging_belt.recall())
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();

        self.frame_counter.frame_presented();

        Ok(())
    }
}
