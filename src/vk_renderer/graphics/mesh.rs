use std::sync::Arc;

use glam::{Mat4, Quat, Vec3};
use vulkano::{buffer::BufferContents, command_buffer::{CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo, CommandBufferInheritanceRenderPassType}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, pipeline::Pipeline, query::QueryPipelineStatisticFlags, render_pass::Framebuffer};

use super::{super::{command::{CommandBufferType, SecondaryCmdBufType, VkBuilder}, pipeline::VkGraphicsPipeline}, camera::Camera};
use super::super::{shaders::graphics_pipeline, buffer::{VkBuffer, VkIterBuffer}, renderer::Renderer, vertex::Vertex, vk::Vk, presenter::FRAMES_IN_FLIGHT};

type Mat = [[f32;4];4];

#[derive(BufferContents, Clone, Copy, Debug)]
#[repr(C)]
pub struct UniformBuffer {
    // #[format(R32G32_SFLOAT)]
    pub model: Mat,
    pub view: Mat,
    pub proj: Mat,
}

impl UniformBuffer {
    pub fn create(vk: Arc<Vk>, model: Mat, view: Mat, proj: Mat) -> VkBuffer<UniformBuffer> {
        let data = UniformBuffer {
            model,
            view,
            proj,
        };
        
        VkBuffer::uniform(vk.allocators.clone(), data)
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
    pub cmds: Vec<Option<SecondaryCmdBufType>>,
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
            cmds: vec![None; *FRAMES_IN_FLIGHT],
        }
    }

    pub fn rebuild(&mut self, vk: Arc<Vk>) {
        self.vbo = VkIterBuffer::vertex(vk.allocators.clone(), self.vertices.to_vec());
        self.ebo = VkIterBuffer::index(vk.allocators.clone(), self.indices.to_vec());
    }

    /* TODO: this as a sec cmd buf */
    pub fn record_command_buffer(
        &mut self, 
        pipeline: &VkGraphicsPipeline, 
        framebuffer: Arc<Framebuffer>, 
        vk: Arc<Vk>,
        camera: &Camera,
        i: usize
    ) {
        let mut builder = VkBuilder::secondary_from_renderpass(vk.clone(), pipeline, framebuffer.clone());

        builder
            .bind_pipeline_graphics(pipeline.graphics_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics, 
                pipeline.graphics_pipeline.layout().clone(), 
                0, 
                self.get_desc_set(vk.clone(), camera, pipeline).0
            )
            .unwrap()
            .bind_vertex_buffers(0, self.vbo.content.clone())
            .unwrap()
            .bind_index_buffer(self.ebo.content.clone())
            .unwrap()
            .draw_indexed(self.ebo.content.len() as u32, 1, 0, 0, 0)
            .unwrap();
            
        self.cmds[i] = Some(builder.build().unwrap());
    }

    pub fn get_model(&self) -> [[f32; 4]; 4] {
        let model_matrix = 
            Mat4::from_translation(self.position) *
            Mat4::from_quat(self.rotation) *
            Mat4::from_scale(self.scale);

        model_matrix.to_cols_array_2d()
    }

    pub fn get_desc_set(&self, vk: Arc<Vk>, camera: &Camera, pipeline: &VkGraphicsPipeline) -> (Arc<PersistentDescriptorSet>, usize) {
        let ubo = UniformBuffer::create(
            vk.clone(),
            self.get_model(),
            camera.get_view(),
            camera.get_proj()
        );

        graphics_pipeline::descriptor_set(
            vk.clone(), 
            0,
            pipeline.graphics_pipeline.clone(), 
            [WriteDescriptorSet::buffer(1, ubo._content)]
        )
    }
}