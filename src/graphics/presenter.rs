use std::sync::Arc;

use crate::graphics::utils::framebuffers_with_depth;

use super::vk::{swapchain, Vk};
use vulkano::{command_buffer::CommandBufferExecFuture, image::Image, render_pass::{Framebuffer, RenderPass}, swapchain::{self, PresentFuture, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{self, future::{FenceSignalFuture, JoinFuture}, GpuFuture}, Validated, VulkanError};
use winit::window::Window;

use super::command::CommandBufferType;

type Fence = Arc<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>>;

 pub struct Presenter {
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub cmd_bufs: Vec<CommandBufferType>,

    pub recreate_swapchain: bool,
    pub window_resized: bool,

    pub fences: Vec<Option<Fence>>,
    pub prev_fence_i: u32,

    pub image_i: usize,
 }

 impl Presenter {
    pub fn new(vk: Arc<Vk>, window: Arc<Window>) -> Self {
        let (swapchain, images) = swapchain(vk.clone(), window);
    
        let frames_in_flight = images.len();

        let fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let prev_fence_i = 0;

        Self {
            swapchain,
            images,
            framebuffers: vec![],        
            cmd_bufs: vec![],

            recreate_swapchain: false,
            window_resized: false,
            fences,
            prev_fence_i,
            image_i: 0,
        }
    }

    pub fn recreate(
        &mut self, 
        vk: Arc<Vk>, 
        rp: Arc<RenderPass>,
        window: Arc<Window>,

    ) {
        if self.window_resized || self.recreate_swapchain {
            self.recreate_swapchain = false;

            let size = window.inner_size();

            let (new_swapchain, new_images) = self.swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: size.into(),
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");
                
            self.swapchain = new_swapchain;
            self.images = new_images;

            self.framebuffers = framebuffers_with_depth(
                vk.clone(),
                rp.clone(),
                &self.images,
            );
            
            if self.window_resized {    
                self.window_resized = false;

                // let viewport = Viewport {
                //     offset: [0.0, 0.0],
                //     extent: size.into(),
                //     depth_range: 0.0..=1.0,
                // };
                
                /*
                here get new pipeline and new command buffers
                 */
            }
        }
    }

    pub fn present(
        &mut self, 
        vk: Arc<Vk>,
    ) { 
        if self.cmd_bufs.len() == 0 { return; }

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
                let mut now = sync::now(vk.device.clone());
                now.cleanup_finished();
                now.boxed()
            }
            Some(fence) => fence.boxed(),
        };

        let future = previous_future
            .join(acquire_future)
            .then_execute(vk.queue.clone(), self.cmd_bufs[image_i as usize].clone())
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