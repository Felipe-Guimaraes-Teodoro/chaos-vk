use std::sync::Arc;

use glam::{Mat4, Quat, Vec3};
use vulkano::{buffer::BufferContents, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}};

use super::super::{shaders::graphics_pipeline, buffer::{VkBuffer, VkIterBuffer}, renderer::Renderer, vertex::Vertex, vk::Vk};

type Mat = [[f32;4];4];

#[derive(BufferContents, Clone, Copy, Debug)]
#[repr(C)]
pub struct UniformBuffer {
    // #[format(R32G32_SFLOAT)]
    pub model:Mat,
}

impl UniformBuffer {
    pub fn create(renderer: &Renderer, model: Mat) -> VkBuffer<UniformBuffer> {
        let data = UniformBuffer {
            model,
        };
        
        VkBuffer::uniform(renderer.vk.allocators.clone(), data)
    }
}

#[derive(Clone)]
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

    pub fn rebuild(&mut self, vk: Arc<Vk>) {
        self.vbo = VkIterBuffer::vertex(vk.allocators.clone(), self.vertices.to_vec());
        self.ebo = VkIterBuffer::index(vk.allocators.clone(), self.indices.to_vec());
    }

    pub fn get_model(&self) -> [[f32; 4]; 4] {
        let model_matrix = 
            Mat4::from_translation(self.position) *
            Mat4::from_quat(self.rotation) *
            Mat4::from_scale(self.scale);

        model_matrix.to_cols_array_2d()
    }

    pub fn get_desc_set(&self, renderer: &Renderer) -> (Arc<PersistentDescriptorSet>, usize) {
        let ubo = UniformBuffer::create(
            renderer, 
            self.get_model(),
        );

        graphics_pipeline::descriptor_set(
            renderer.vk.clone(), 
            0,
            renderer.presenter.pipelines[0].graphics_pipeline.clone(), 
            [WriteDescriptorSet::buffer(1, ubo._content)]
        )
    }
}