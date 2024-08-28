use std::{future::IntoFuture, sync::Arc};

use vulkano::{buffer::Subbuffer, command_buffer::{allocator::{StandardCommandBufferAlloc, StandardCommandBufferAllocator}, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo}, image::Image, pipeline::GraphicsPipeline, render_pass::Framebuffer, swapchain::{self, acquire_next_image, acquire_next_image_raw, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{self, GpuFuture}, Validated, VulkanError};
use winit::{event_loop::EventLoop, window::{self, Window, WindowBuilder}};

use crate::vk_renderer::{buffer::VkIterBuffer, pipeline::VkGraphicsPipeline, shaders::{fragment_shader, graphics_pipeline::{framebuffers, render_pass}, vertex_shader}, swapchain, Vk};

use super::{command::{CommandBufferType, VkBuilder}, geometry::fundamental::circle, shaders::graphics_pipeline::{self, framebuffer}, vertex::{self, Vertex}};

pub struct Renderer {
    pub vk: Arc<Vk>,

    pub presenter: Presenter,
    
    // meshes: Vec<Mesh>,
} 

impl Renderer {
    pub fn new(el: &EventLoop<()>) -> Self {
        let vk = Arc::new(Vk::new(&el));

        let pr = Presenter::new(vk.clone(), el);

        Self {
            vk,
            presenter: pr
        }
    }

    pub fn update(&mut self, vert_buffer: Subbuffer<[Vertex]>) {
        self.presenter.update(self.vk.clone(), vert_buffer);
    }
 }


 /* todo: move presenter to its own file */
 pub struct Presenter {
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub pipeline: VkGraphicsPipeline,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,

    pub recreate_swapchain: bool,
    pub window_resized: bool,
 }

 impl Presenter {
    pub fn new(vk: Arc<Vk>, el: &EventLoop<()>) -> Self {
        let (swapchain, images) = swapchain(vk.clone());

        let pipeline = VkGraphicsPipeline::new(
            vk.clone(), 
            vertex_shader::load(vk.device.clone()).unwrap(), 
            fragment_shader::load(vk.device.clone()).unwrap(), 
            Some(swapchain.clone())
        );

        let framebuffers = framebuffers(
            render_pass( vk.clone(), Some(swapchain.clone()) ),
            &images.clone()
        );

        Self {
            swapchain,
            images,
            pipeline,
            framebuffers,        
            command_buffers: vec![],

            recreate_swapchain: false,
            window_resized: false,
        }
    }
    
    pub fn get_command_buffers(
        &self, 
        vk: Arc<Vk>, 
        content: Subbuffer<[Vertex]>,
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
                    .unwrap()
                    .bind_vertex_buffers(0, content.clone())
                    .unwrap()
                    .draw(content.len() as u32, 1, 0, 0)
                    .unwrap()
                    .end_render_pass(Default::default())
                    .unwrap();

                builder.build().unwrap()
            })
            .collect()
    }

    pub fn update(&mut self, vk: Arc<Vk>, content: Subbuffer<[Vertex]>) {
        if self.recreate_swapchain || self.window_resized {
            self.recreate_swapchain = false;
    
            let new_dimensions = vk.window.inner_size();
    
            let (new_swapchain, new_images) = self.swapchain
                .recreate(SwapchainCreateInfo {
                    // Here, `image_extend` will correspond to the window dimensions.
                    image_extent: new_dimensions.into(),
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain: {e}");
            self.swapchain = new_swapchain;
            let new_framebuffers = framebuffers(self.pipeline.render_pass.clone(), &new_images);

            if self.window_resized {
                self.window_resized = false;

                self.pipeline.viewport.extent = new_dimensions.into();
                self.pipeline.graphics_pipeline = 
                    graphics_pipeline::graphics_pipeline(
                        vk.clone(),
                        self.pipeline.vs.clone(),
                        self.pipeline.fs.clone(),
                        self.pipeline.render_pass.clone(),
                        self.pipeline.viewport.clone(),
                    );
            
                // here lies the issue
                // self.command_buffers = self.get_command_buffers(vk, content);
            }
        }
    }

    pub fn present(&mut self, vk: Arc<Vk>) {
        let (image_i, suboptimal, acquire_future) = 
            match swapchain::acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };
        
        if suboptimal {
            self.recreate_swapchain = true;
        }

        let execution = sync::now(vk.device.clone())
            .join(acquire_future)
            .then_execute(vk.queue.clone(), self.command_buffers[image_i as usize].clone())
            .unwrap()
            .then_swapchain_present(
                vk.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_i),
            )
            .then_signal_fence_and_flush();

        match execution.map_err(Validated::unwrap) {
            Ok(future) => {
                future.wait(None).unwrap();
            },
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
            },
            Err(e) => {
                println!("failed to flush future: {e}");
            }
        }
    }
 }