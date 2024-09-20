use glam::Vec3;
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex as VulkanoVertex};

#[derive(BufferContents, VulkanoVertex, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub pos: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub col: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub norm: [f32; 3],
}

impl Vertex {
    pub fn new(vec: Vec3, norm: Vec3) -> Self {
        Self {
            pos: [vec.x, vec.y, vec.z],
            col: [1.0, 1.0, 1.0],
            norm: [norm.x, norm.y, norm.z]
        }
    }
}

