use std::sync::Arc;

use vulkano::{command_buffer::{allocator::StandardCommandBufferAllocator, CommandBufferExecFuture, PrimaryAutoCommandBuffer}, image::Image, render_pass::Framebuffer, swapchain::{self, PresentFuture, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{future::{FenceSignalFuture, JoinFuture}, now, GpuFuture}, Validated, VulkanError};

use super::{events::event_loop::EventLoop, pipeline::VkGraphicsPipeline, shaders::{fragment_shader, graphics_pipeline::{framebuffers, render_pass}, vertex_shader}, swapchain, Vk};


type Fence = Arc<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>>;

 pub struct Presenter {
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub pipeline: VkGraphicsPipeline,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,

    pub recreate_swapchain: bool,
    pub window_resized: bool,

    pub fences: Vec<Option<Fence>>,
    pub prev_fence_i: u32,
 }

 impl Presenter {
    pub fn new(vk: Arc<Vk>, el: &EventLoop) -> Self {
        let (swapchain, images) = swapchain(vk.clone(), el);

        let pipeline = VkGraphicsPipeline::new(
            vk.clone(), 
            vertex_shader::load(vk.device.clone()).unwrap(), 
            fragment_shader::load(vk.device.clone()).unwrap(), 
            Some(swapchain.clone())
        );

        let framebuffers = framebuffers(
            vk.clone(),
            render_pass( vk.clone(), Some(swapchain.clone()) ),
            &images.clone()
        );
    
        let frames_in_flight = 3;
        let fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let prev_fence_i = 0;

        Self {
            swapchain,
            images,
            pipeline,
            framebuffers,        
            command_buffers: vec![],

            recreate_swapchain: false,
            window_resized: false,
            fences,
            prev_fence_i,
        }
    }

    pub fn update(&mut self, vk: Arc<Vk>, el: &EventLoop) {
        if self.window_resized || self.recreate_swapchain {
            self.recreate_swapchain = false;

            let (new_w, new_h) = (el.event_handler.width, el.event_handler.height);

            let (new_swapchain, new_images) = self.swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: [new_w as u32, new_h as u32],
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            self.pipeline.viewport.extent = [new_w, new_h];

            self.framebuffers = framebuffers(
                vk.clone(),
                self.pipeline.render_pass.clone(),
                &new_images
            );

            if self.window_resized {
                self.window_resized = false;
                
                self.pipeline.viewport.extent = [new_w, new_h];
                self.pipeline.graphics_pipeline = crate::vk_renderer::shaders::graphics_pipeline::graphics_pipeline(
                    vk.clone(),
                    self.pipeline.vs.clone(),
                    self.pipeline.fs.clone(),
                    self.pipeline.render_pass.clone(),
                    self.pipeline.viewport.clone(),
                );
            }

            self.swapchain = new_swapchain;
        }

        let (image_i, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
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

        if let Some(image_fence) = &self.fences[image_i as usize] {
            image_fence.wait(None).unwrap();
        }

        let previous_future = match self.fences[self.prev_fence_i as usize].clone() {
            None => {
                let mut now = now(vk.device.clone());
                now.cleanup_finished();

                now.boxed()
            }
            Some(fence) => vulkano::sync::GpuFuture::boxed(fence),
        };

        let future = previous_future
            .join(acquire_future)
            .then_execute(vk.queue.clone(), self.command_buffers[image_i as usize].clone())
            .unwrap()
            .then_swapchain_present(
                vk.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_i),
            )
            .then_signal_fence_and_flush();

        self.fences[image_i as usize] = match future.map_err(Validated::unwrap) {
            Ok(value) => Some(Arc::new(value)),
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                None
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                None
            }
        };

        self.prev_fence_i = image_i;
    }
 }