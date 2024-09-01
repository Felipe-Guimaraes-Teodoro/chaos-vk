use std::sync::Arc;

use vulkano::{buffer::IndexBuffer, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, descriptor_set::WriteDescriptorSet, pipeline::{GraphicsPipeline, Pipeline}, render_pass::Framebuffer};
use winit::event_loop::EventLoop;

use crate::vk_renderer::{buffer::VkIterBuffer, Vk};

use super::{graphics::{camera::Camera, mesh::{Mesh, UniformBuffer}}, presenter::Presenter, shaders::graphics_pipeline, vertex::Vertex};

pub struct Renderer {
    pub vk: Arc<Vk>,

    pub camera: Camera,
    
    pub meshes: Vec<Mesh>,
} 

impl Renderer {
    pub fn new(el: &EventLoop<()>) -> Self {
        let vk = Arc::new(Vk::new(&el));

        let camera = Camera::new();

        Self {
            vk,
            camera,
            meshes: vec![],
        }
    }

    pub fn get_command_buffers(
        vk: Arc<Vk>, 
        renderer: &Renderer,
        pipeline: Arc<GraphicsPipeline>,
        framebuffers: Vec<Arc<Framebuffer>>,
    ) -> Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>> {
        let (view, proj) = (renderer.camera.get_view(), renderer.camera.get_proj());

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
                    let ubo = UniformBuffer::create(
                        renderer, 
                        mesh.get_model(), 
                        view,
                        proj,
                    );

                    let (descriptor_set, idx) = graphics_pipeline::descriptor_set(
                        vk.clone(), 
                        pipeline.clone(), 
                        [WriteDescriptorSet::buffer(0, ubo._content.clone())]
                    );

                    builder
                        .bind_descriptor_sets(
                            vulkano::pipeline::PipelineBindPoint::Graphics, 
                            pipeline.layout().clone(), 
                            0, 
                            descriptor_set
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

