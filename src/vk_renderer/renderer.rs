use std::{collections::HashMap, sync::Arc};

use image::Frame;
use vulkano::{buffer::IndexBuffer, command_buffer::{RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, pipeline::Pipeline, render_pass::Framebuffer};

use crate::vk_renderer::Vk;

use super::{command::{CommandBufferType, VkBuilder}, events::event_loop::EventLoop, graphics::{camera::Camera, mesh::Mesh}, pipeline::VkGraphicsPipeline, presenter::Presenter, shaders::renderpass::VkSecRenderpass};

pub struct Renderer {
    pub vk: Arc<Vk>,
    pub presenter: Presenter,
    pub camera: Camera,
    pub meshes: HashMap<usize, Mesh>,

    pub sec_renderpasses: Vec<VkSecRenderpass>,
    pub img_idx: usize,
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
            img_idx: 0,
        }
    }

    pub fn get_command_buffers(
        &mut self,
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
                        contents: SubpassContents::SecondaryCommandBuffers,
                        ..Default::default()
                    },
                )
                .unwrap();
                // .bind_pipeline_graphics(self.presenter.pipelines[0].graphics_pipeline.clone())
                // .unwrap();

            self.append_mesh_sec_renderpass_to_builder(&mut builder, framebuffer.clone());

            builder.0
                .end_render_pass(Default::default())
                .unwrap();
            
            self.append_sec_renderpasses_to_builder(&mut builder);

            cmd_bufs.push(builder.0.build().unwrap());

            self.img_idx += 1;
        }

        cmd_bufs 
    }

    pub fn update(&mut self, el: &mut EventLoop) {
        // self.presenter.recreate_swapchain = true;
        let cmd_bufs = self.get_command_buffers(
            self.presenter.framebuffers.clone(),
        );
        self.presenter.cmd_bufs = cmd_bufs;
        self.presenter.update(self.vk.clone(), el);
    }

    pub fn append_sec_renderpasses_to_builder(&mut self, builder: &mut VkBuilder) {
        for pass in &self.sec_renderpasses {
            builder.0
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: pass.clear_values.clone(),
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
    }

    /* TODO: pipeline as a membr of mesh; or renderer, then render sorted on the pipeline
    for pipeline in pipelines {
        bind pipeline

        for mesh in pipeline.mesh {
            execute commands
        }
    }

    ALSO: try the other type of secondary command buffer
    (it might be useful in this case)
    */
    pub fn append_mesh_sec_renderpass_to_builder(
        &mut self, 
        builder: &mut VkBuilder, 
        framebuffer: Arc<Framebuffer>,
    ) {
        let frame_i = self.img_idx % 3;

        for mesh in self.meshes.values_mut() {
            /* 
            if let Some(cmd) = &mesh.cmds[frame_i] { 
                /* check  framebuffer! */
                dbg!("i'm tryna draw something");
                builder.0
                    .execute_commands(cmd.clone())
                    .unwrap();
                continue;
                
            }
            
            mesh.record_command_buffer(
                &self.presenter.pipelines[0].clone(),
                framebuffer.clone(),
                self.vk.clone(),
                &self.camera,
                frame_i
            );
            */
            
            mesh.record_command_buffer(
                &self.presenter.pipelines[0].clone(), /* this could be a member of the mesh struct */
                framebuffer.clone(),
                self.vk.clone(),
                &self.camera,
                frame_i
            );

            builder.0.execute_commands(
                mesh.cmds[frame_i].clone().unwrap()
            ).unwrap();
            
        }
    }
    
    
 }

