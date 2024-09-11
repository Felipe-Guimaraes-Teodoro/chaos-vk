use std::sync::Arc;

use vulkano::{buffer::{Buffer, IndexBuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, descriptor_set::{DescriptorSet, WriteDescriptorSet}, pipeline::{GraphicsPipeline, Pipeline}, render_pass::Framebuffer};

use crate::vk_renderer::Vk;

use super::{command::VkBuilder, events::event_loop::EventLoop, graphics::{camera::Camera, mesh::{Mesh, UniformBuffer}}, presenter::Presenter, shaders::graphics_pipeline::{self, descriptor_set}};

pub struct Renderer {
    pub vk: Arc<Vk>,

    pub presenter: Presenter,

    pub camera: Camera,
    
    pub meshes: Vec<Mesh>,
} 

impl Renderer {
    pub fn new(el: &mut EventLoop) -> Self {
        let vk = Arc::new(Vk::new(el));

        let camera = Camera::new();

        let presenter = Presenter::new(vk.clone(), el);

        Self {
            vk,
            presenter,
            camera,
            meshes: vec![],
        }
    }

    pub fn get_command_buffers(
        &self,
        pipeline: Arc<GraphicsPipeline>,
        framebuffers: Vec<Arc<Framebuffer>>,
    ) -> Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>> {
        let (view, proj) = (self.camera.get_view(), self.camera.get_proj());

        let command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>> = framebuffers
            .iter()
            .map(|framebuffer| {
                let mut builder = VkBuilder::new_multiple(self.vk.clone());

                builder.0
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.1, 0.2, 0.3, 1.0].into()), Some(1.0.into())],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .bind_pipeline_graphics(pipeline.clone())
                    .unwrap();

                for mesh in &self.meshes {
                    builder.0
                        .bind_descriptor_sets(
                            vulkano::pipeline::PipelineBindPoint::Graphics, 
                            pipeline.layout().clone(), 
                            0, 
                            mesh.get_desc_set(&self).0
                        )
                        .unwrap()
                        .bind_vertex_buffers(0, mesh.vbo.content.clone())
                        .unwrap()
                        .bind_index_buffer(IndexBuffer::U32(mesh.ebo.content.clone()))
                        .unwrap()
                        .draw_indexed(
                            mesh.ebo.content.len() as u32, 
                            1, 
                            0, 
                            0, 
                            0
                        )
                        .unwrap();
                }

                builder.0
                    .end_render_pass(Default::default())
                    .unwrap();

                builder.0.build().unwrap()
            })
            .collect();

;        command_buffers
    }

    /* todo: on presenter.update have renderer update the command buffers and send it as an argument to presenter.update instead of getting the commandbuffers from renderer otseçf */
    pub fn update(&mut self, el: &mut EventLoop) {
        // self.presenter.recreate_swapchain = true;
        self.presenter.command_buffers = self.get_command_buffers(
            self.presenter.pipeline.graphics_pipeline.clone(), 
            self.presenter.framebuffers.clone(),
        );
        self.presenter.update(self.vk.clone(), el);
    }
 }

