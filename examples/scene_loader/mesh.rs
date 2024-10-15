use std::sync::Arc;

use chaos_vk::{graphics::{buffer::{VkBuffer, VkIterBuffer}, command::BuilderType, utils::descriptor_set, vertex::{InstanceData, RVertex}, vk::Vk}, util::math::{rand_betw, rand_vec3}};
use glam::{Mat4, Quat, Vec3};
use vulkano::{descriptor_set::WriteDescriptorSet, pipeline::{GraphicsPipeline, Pipeline}};

use super::shaders::vs;

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<RVertex>,
    pub indices: Vec<u32>,
    pub instances: Vec<InstanceData>,

    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub color: Vec3,

    pub vbo: VkIterBuffer<RVertex>,
    pub ibo: VkIterBuffer<InstanceData>,
    pub ebo: VkIterBuffer<u32>,
}

impl Mesh {
    pub fn new(vk: Arc<Vk>, vertices: &Vec<RVertex>, indices: &Vec<u32>) -> Self {
        let instances = vec![InstanceData {ofs: [0.0, 0.0, 0.0]}];

        Self {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            instances: instances.to_vec(),

            position: Vec3::ZERO,
            rotation: Quat::default(),
            scale: Vec3::ONE,
            color: Vec3::ONE,

            vbo: VkIterBuffer::vertex(vk.allocators.clone(), vertices.to_vec()),
            ebo: VkIterBuffer::index(vk.allocators.clone(), indices.to_vec()),
            ibo: VkIterBuffer::vertex(vk.allocators.clone(), instances),
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

    pub fn get_ubo(&self, vk: Arc<Vk>) -> VkBuffer<vs::Model> {
        VkBuffer::uniform(vk.allocators.clone(), vs::Model {
            model: self.get_model(),
        })
    }

    /// Warning: this function assumes a graphics pipeline has already been bounded
    pub fn build_commands(&self, vk: Arc<Vk>, builder: &mut BuilderType, pipeline: Arc<GraphicsPipeline>) {
        let ubo = self.get_ubo(vk.clone());
        builder
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics, 
                pipeline.layout().clone(), 
                1, 
                descriptor_set(
                    vk.clone(), 
                    1,
                    pipeline.clone(), 
                    [WriteDescriptorSet::buffer(0, ubo.content.clone())]
                ).0
            ).unwrap()
            .bind_vertex_buffers(0, 
                (self.vbo.content.clone(), self.ibo.content.clone())
            )
            .unwrap()
            .bind_index_buffer(self.ebo.content.clone())
            .unwrap()
            .draw_indexed(self.ebo.content.len() as u32, self.ibo.content.len() as u32, 0, 0, 0)
            .unwrap();
    }
}
