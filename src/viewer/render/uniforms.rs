use super::Camera;
use cgmath::InnerSpace;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Uniforms {
    view_proj: cgmath::Matrix4<f32>,
    // We use Vector4 instead of Vector3 due to GLSL block alignments
    // See https://stackoverflow.com/questions/35524814/
    light_pos: cgmath::Vector4<f32>,
    light_color: cgmath::Vector4<f32>,
}

impl Uniforms {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity(),
            light_pos: cgmath::Vector4::unit_x(),
            light_color: (1.0, 1.0, 1.0, 1.0).into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix();
        let back = camera.eye();
        let up = camera.up();
        let left = up.cross(back);
        self.light_pos = (back + up + left).normalize_to(20.0).extend(1.0);
    }
}
