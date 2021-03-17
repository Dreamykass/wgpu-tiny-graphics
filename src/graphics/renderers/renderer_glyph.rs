use crate::graphics::{CurrentFrame, GraphicsState};
use rand::Rng;

pub struct RendererGlyph {
    //
}

impl RendererGlyph {
    pub fn new(_graphics_state: &GraphicsState) -> Self {
        RendererGlyph {}
    }

    pub fn draw(&mut self, current_frame: &mut CurrentFrame) {
        let window_inner_size = current_frame.graphics_state.window_inner_size();

        let inconsolata = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../../Inconsolata-Regular.ttf"
        ))
        .unwrap();

        let mut glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(inconsolata).build(
            &current_frame.graphics_state.device,
            current_frame.graphics_state.swap_chain_descriptor.format,
        );

        glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (30.0, 30.0),
            bounds: (
                window_inner_size.width as f32,
                window_inner_size.height as f32,
            ),
            text: vec![wgpu_glyph::Text::new("Hello wgpu_glyph!")
                .with_color([0.0, 1.0, 0.0, 1.0])
                .with_scale(40.0)],
            ..wgpu_glyph::Section::default()
        });

        glyph_brush.queue(wgpu_glyph::Section {
            screen_position: (30.0, 90.0),
            bounds: (
                window_inner_size.width as f32,
                window_inner_size.height as f32,
            ),
            text: vec![wgpu_glyph::Text::new(&*format!(
                "Hello wgpu_glyph! Random number: {}",
                rand::thread_rng().gen_range(0..100)
            ))
            .with_color([1.0, 1.0, 1.0, 1.0])
            .with_scale(40.0)],
            ..wgpu_glyph::Section::default()
        });

        glyph_brush
            .draw_queued(
                &current_frame.graphics_state.device,
                &mut current_frame.graphics_state.staging_belt,
                &mut current_frame.encoder,
                &current_frame.frame.output.view,
                window_inner_size.width,
                window_inner_size.height,
            )
            .expect("Draw queued");

        // current_frame.graphics_state.staging_belt.finish();
    }
}
