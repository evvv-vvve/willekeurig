use super::camera::{Camera, Projection};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        
        Self {
            view_proj: cgmath::Matrix4::identity().into()
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        // update view_position later with the light stuff
        // self.view_position = camera.position.to_homogeneous;
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}