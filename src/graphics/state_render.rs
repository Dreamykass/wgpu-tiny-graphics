use crate::graphics::state::GraphicsState;
use crate::graphics::CurrentFrame;
use crate::vertex::Vertex;
use rand::Rng;
use std::time::Instant;

// new render
impl GraphicsState {
    pub fn begin_current_frame(&mut self) -> Result<CurrentFrame, wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        Ok(CurrentFrame {
            graphics_state: self,
            frame,
            encoder,
        })
    }
}

// old render
impl GraphicsState {
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?;

        // Encodes a series of GPU operations.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // triangle (1st render pass)
        let time = Instant::now();
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
        log::info!("triangle: {:?}ms", time.elapsed().as_secs_f32() * 1000.0);

        // glyphs (2nd internal render pass)
        // let time = Instant::now();
        // self.render_pass_glyph(&mut encoder, &frame);
        // log::info!("glyphs: {:?}ms", time.elapsed().as_secs_f32() * 1000.0);

        // imgui (3rd internal render pass)
        // let time = Instant::now();
        // self.render_pass_imgui(&mut encoder, &frame);
        // log::info!("imgui: {:?}ms", time.elapsed().as_secs_f32() * 1000.0);

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
