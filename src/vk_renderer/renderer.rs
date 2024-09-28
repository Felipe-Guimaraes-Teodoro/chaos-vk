use std::sync::{Arc, Mutex};

use imgui::{Context, DrawData};
use vulkano::{buffer::IndexBuffer, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SecondaryAutoCommandBuffer, SubpassBeginInfo, SubpassContents}, pipeline::Pipeline, render_pass::Framebuffer};

use crate::vk_renderer::Vk;

use super::{command::{BuilderType, CommandBufferType, SecondaryCmdBufType, VkBuilder}, events::event_loop::EventLoop, graphics::{camera::Camera, mesh::Mesh}, pipeline::VkGraphicsPipeline, presenter::Presenter, ui::renderer::ImRenderer};

pub struct Renderer {
    pub vk: Arc<Vk>,
    pub presenter: Presenter,
    pub camera: Camera,
    pub meshes: Vec<Mesh>,

    pub sec_cmd_bufs: Vec<SecondaryCmdBufType>,
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
            sec_cmd_bufs: vec![],
        }
    }

    pub fn get_command_buffers(
        &mut self,
        pipelines: Vec<VkGraphicsPipeline>,
        framebuffers: Vec<Arc<Framebuffer>>,
    ) -> Vec<CommandBufferType> {
        framebuffers
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
                    .bind_pipeline_graphics(pipelines[0].graphics_pipeline.clone())
                    .unwrap();
                    //.push_constants(
                    //    pipelines[0].graphics_pipeline.layout().clone(), 
                    //    0, 
                    //    view,
                    //)
                    //.unwrap()
                    //.push_constants(
                    //    pipelines[0].graphics_pipeline.layout().clone(), 
                    //    size_of::<[[f32; 4]; 4]>() as u32, 
                    //    proj,
                    //)
                    //.unwrap();

                for mesh in &self.meshes {
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

                /* SECONDARY RENDER PASS */
                /* fix for this: 
                    instead of recording a vec of each secondary command
                    buffer, create a new structure VkRenderPass that can
                    store the secondary command buffer and the necessary
                    information for creating the render pass (render_pas
                    s and framebuffer).

                    later iterate through each of those and blah blah bl
                    ah, you get it. bro why is this getting more complic
                    aded each passing second....
                 */
                // builder.0
                //     .begin_render_pass(RenderPassBeginInfo {
                //         render_pass: todo!(),
                //         framebuffer: todo!(),
                //     },
                //     SubpassBeginInfo {
                //         contents: SubpassContents::SecondaryCommandBuffers,
                //         ..Default::default()
                //     }
                // )
// 
                // for sec_cmd_buf in &self.sec_cmd_bufs {
                //     builder.0
                //         .execute_commands(sec_cmd_buf.clone())
                //         .unwrap();
                // }
// 
                // builder.0
                //     .end_render_pass(Default::default())
                //     .unwrap();
                
                builder.0.build().unwrap()
            })
            .collect()
    }

    pub fn update(&mut self, el: &mut EventLoop) {
        // self.presenter.recreate_swapchain = true;
        let cmd_buffer_builders = self.get_command_buffers(
            self.presenter.pipelines.clone(), 
            self.presenter.framebuffers.clone(),
        );
        self.presenter.cmd_bufs = cmd_buffer_builders;
        self.presenter.update(self.vk.clone(), el);
    }
 }

