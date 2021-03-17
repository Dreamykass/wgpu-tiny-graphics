// reimplementation of <https://www.sfml-dev.org/documentation/2.5.1/classsf_1_1View.php>
pub struct SfView {
    pub center: (f32, f32),
    pub size: (f32, f32),
    pub rotation: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX4: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl SfView {
    pub fn get_matrix4(&self) -> cgmath::Matrix4<f32> {
        // https://github.com/SFML/SFML/blob/master/src/SFML/Graphics/View.cpp#L198

        use cgmath::Angle;

        let rotation = self.rotation;
        let center = self.center;
        let size = self.size;

        let angle = rotation * std::f32::consts::PI / 180f32; // in radians now
        let cosine = cgmath::Rad::cos(cgmath::Rad(angle));
        let sine = cgmath::Rad::sin(cgmath::Rad(angle));
        let tx = -center.0 * cosine - center.1 * sine + center.0;
        let ty = center.0 * sine - center.1 * cosine + center.1;

        let a = 2f32 / size.0;
        let b = -2f32 / size.1;
        let c = -a * center.0;
        let d = -b * center.1;

        // note: SFML matrices are row-major, but cgmath is column-major
        #[rustfmt::skip]
        let m4 = cgmath::Matrix4::new(
            a*cosine,       -b*sine,        0f32,        0f32,
            a*sine,          b*cosine,      0f32,        0f32,
            0f32,            0f32,          1f32,        0f32,
            a*tx+c,          b*ty+d,        0f32,        1f32,
        );

        m4
    }
}
