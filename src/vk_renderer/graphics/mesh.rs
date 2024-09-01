use std::sync::Arc;

use glam::{Mat4, Quat, Vec3};
use vulkano::{buffer::{BufferContents, IndexBuffer}, shader::ShaderModule};

use crate::vk_renderer::{buffer::{VkBuffer, VkIterBuffer}, renderer::Renderer, vertex::Vertex};

type Mat = [[f32;4];4];

#[derive(BufferContents, Clone, Copy)]
#[repr(C)]
pub struct UniformBuffer {
    // #[format(R32G32_SFLOAT)]
    pub model: Mat,
    pub view: Mat,
    pub proj: Mat,
}

impl UniformBuffer {
    pub fn create(renderer: &Renderer, model: Mat, view: Mat, proj: Mat) -> VkBuffer<UniformBuffer> {
        let data = UniformBuffer {
            model,
            view,
            proj,
        };
        
        VkBuffer::new(renderer.vk.allocators.clone(), data)
    }
}


pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub color: Vec3,

    pub vbo: VkIterBuffer<Vertex>,
    pub ebo: VkIterBuffer<u32>,
}

impl Mesh {
    pub fn new(vertices: &Vec<Vertex>, indices: &Vec<u32>, renderer: &Renderer) -> Self {
        Self {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),

            position: Vec3::ZERO,
            rotation: Quat::default(),
            scale: Vec3::ONE,
            color: Vec3::ONE,

            vbo: VkIterBuffer::vertex(renderer.vk.allocators.clone(), vertices.to_vec()),
            ebo: VkIterBuffer::index(renderer.vk.allocators.clone(), indices.to_vec()),
        }
    }

    pub fn get_model(&self) -> [[f32; 4]; 4] {
        let model_matrix = 
            Mat4::from_translation(self.position) *
            Mat4::from_quat(self.rotation) *
            Mat4::from_scale(self.scale);

        model_matrix.to_cols_array_2d()
    }
}