use crate::graphics::state::GraphicsState;
use rand::Rng;

impl GraphicsState {
    pub fn render_pass_glyph(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::SwapChainFrame,
    ) {
        let window_inner_size = self.window_inner_size();

        let inconsolata = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../Inconsolata-Regular.ttf"
        ))
        .unwrap();

        let mut glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(inconsolata)
            .build(&self.device, self.swap_chain_descriptor.format);

        glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (30.0, 30.0),
            bounds: (
                self.window_inner_size().width as f32,
                self.window_inner_size().height as f32,
            ),
            text: vec![wgpu_glyph::Text::new("Hello wgpu_glyph!")
                .with_color([0.0, 1.0, 0.0, 1.0])
                .with_scale(40.0)],
            ..wgpu_glyph::Section::default()
        });

        glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (30.0, 90.0),
            bounds: (
                self.window_inner_size().width as f32,
                self.window_inner_size().height as f32,
            ),
            text: vec![wgpu_glyph::Text::new(&*format!(
                "Hello wgpu_glyph! {}",
                rand::thread_rng().gen_range(0..100)
            ))
            .with_color([1.0, 1.0, 1.0, 1.0])
            .with_scale(40.0)],
            ..wgpu_glyph::Section::default()
        });

        glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                encoder,
                &frame.output.view,
                window_inner_size.width,
                window_inner_size.height,
            )
            .expect("Draw queued");

        self.staging_belt.finish();
    }
}
