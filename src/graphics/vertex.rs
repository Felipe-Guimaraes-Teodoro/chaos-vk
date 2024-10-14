use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};


#[derive(BufferContents, Vertex, Copy, Clone)]
#[repr(C)]
pub struct RVertex {
    #[format(R32G32B32_SFLOAT)]
    pub pos: [f32; 3],
}