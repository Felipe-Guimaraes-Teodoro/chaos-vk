use std::{collections::HashMap, sync::{Arc, Mutex}};

use imgui::{Context, DrawData};
use vulkano::{buffer::IndexBuffer, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SecondaryAutoCommandBuffer, SubpassBeginInfo, SubpassContents}, descriptor_set::PersistentDescriptorSet, pipeline::Pipeline, render_pass::{Framebuffer, Subpass}};

use crate::vk_renderer::Vk;

use super::{command::{BuilderType, CommandBufferType, SecondaryCmdBufType, VkBuilder}, events::event_loop::EventLoop, graphics::{camera::Camera, mesh::Mesh}, pipeline::VkGraphicsPipeline, presenter::Presenter, shaders::renderpass::VkSecRenderpass, ui::renderer::ImRenderer};

pub struct Renderer {
    pub vk: Arc<Vk>,
    pub presenter: Presenter,
    pub camera: Camera,
    pub meshes: HashMap<usize, Mesh>,

    pub sec_renderpasses: Vec<VkSecRenderpass>,
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
            meshes: HashMap::new(),
            sec_renderpasses: vec![],
        }
    }

    pub fn get_command_buffers(
        &mut self,
        pipelines: Vec<VkGraphicsPipeline>,
        framebuffers: Vec<Arc<Framebuffer>>,
    ) -> Vec<CommandBufferType> {
        let mut cmd_bufs = vec![];

        for framebuffer in framebuffers {
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
                .bind_pipeline_graphics(pipelines[0].graphics_pipeline.clone())
                .unwrap();

            for mesh in self.meshes.values() {
                builder.0
                    .bind_descriptor_sets(
                        vulkano::pipeline::PipelineBindPoint::Graphics, 
                        pipelines[0].graphics_pipeline.layout().clone(), 
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
            
            for pass in &self.sec_renderpasses {
                builder.0
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![None], /* todo: add this field to vksecrenderpass */
                            ..RenderPassBeginInfo::framebuffer(pass.framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::SecondaryCommandBuffers,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                
                builder.0
                    .execute_commands(pass.cmd_buf.clone())
                    .unwrap();

                builder.0
                    .end_render_pass(Default::default())
                    .unwrap();
            }

            self.sec_renderpasses.clear();

            cmd_bufs.push(builder.0.build().unwrap());
        }

        cmd_bufs 
    }

    pub fn update(&mut self, el: &mut EventLoop) {
        // self.presenter.recreate_swapchain = true;
        let cmd_bufs = self.get_command_buffers(
            self.presenter.pipelines.clone(), 
            self.presenter.framebuffers.clone(),
        );
        self.presenter.cmd_bufs = cmd_bufs;
        self.presenter.update(self.vk.clone(), el);
    }
 }

