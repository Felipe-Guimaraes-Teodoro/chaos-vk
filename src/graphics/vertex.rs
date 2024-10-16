use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};


#[derive(BufferContents, Vertex, Copy, Clone)]
#[repr(C)]
pub struct PosVertex {
    #[format(R32G32B32_SFLOAT)]
    pub pos: [f32; 3],
}

#[derive(BufferContents, Clone, Copy, Vertex)]
#[repr(C)]
pub struct InstanceData {
    #[format(R32G32B32_SFLOAT)]
    pub ofs: [f32; 3],
}