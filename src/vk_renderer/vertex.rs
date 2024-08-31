use glam::Vec3;
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex as VulkanoVertex};

#[derive(BufferContents, VulkanoVertex, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    pub pos: [f32; 2],
}

impl Vertex {
    pub fn from_vec(vec: Vec3) -> Self {
        Self {
            pos: [vec.x, vec.y],
        }
    }
}

