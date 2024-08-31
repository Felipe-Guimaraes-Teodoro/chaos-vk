use std::sync::Arc;

use vulkano::{buffer::IndexBuffer, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, pipeline::{GraphicsPipeline, Pipeline}, render_pass::Framebuffer};
use winit::event_loop::EventLoop;

use crate::vk_renderer::{buffer::VkIterBuffer, Vk};

use super::{graphics::mesh::Mesh, presenter::Presenter, vertex::Vertex};

pub struct Renderer {
    pub vk: Arc<Vk>,
    
    pub meshes: Vec<Mesh>,
} 

impl Renderer {
    pub fn new(el: &EventLoop<()>) -> Self {
        let vk = Arc::new(Vk::new(&el));

        Self {
            vk,
            meshes: vec![],
        }
    }

    pub fn get_command_buffers(
        vk: Arc<Vk>, 
        renderer: &Renderer,
        pipeline: Arc<GraphicsPipeline>,
        framebuffers: Vec<Arc<Framebuffer>>,
    ) -> Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>> {
        framebuffers
            .iter()
            .map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    &vk.allocators.command,
                    vk.queue.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                )
                .unwrap();

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
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

                for mesh in &renderer.meshes {
                    builder
                        .bind_descriptor_sets(
                            vulkano::pipeline::PipelineBindPoint::Graphics, 
                            pipeline.layout().clone(), 
                            0, 
                            
                        )
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

                builder
                    .end_render_pass(Default::default())
                    .unwrap();
                    /* 
                builder
                    .bind_vertex_buffers(0, thing.0.content.clone())
                    .unwrap()
                    .bind_index_buffer(thing.1.clone())
                    .unwrap()
                    .draw_indexed(thing.1.len() as u32, 1, 0, 0, 0)
                    .unwrap()
                    .end_render_pass(Default::default())
                    .unwrap();
                    */

                builder.build().unwrap()
            })
            .collect()
    }

    /* todo: on presenter.update have renderer update the command buffers and send it as an argument to presenter.update instead of getting the commandbuffers from renderer otseçf */
    pub fn update(&mut self) {
    }
 }

