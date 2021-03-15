use crate::graphics::CurrentFrame;

pub struct RenderPass<'a> {
    // pub(super) current_frame: &'a mut CurrentFrame<'a>,
    pub(super) render_pass: wgpu::RenderPass<'a>,
}

impl<'a> RenderPass<'a> {
    pub fn new(current_frame: &'a mut CurrentFrame<'a>) -> Self {
        let render_pass = current_frame
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

        RenderPass {
            // current_frame,
            render_pass,
        }
    }
}
