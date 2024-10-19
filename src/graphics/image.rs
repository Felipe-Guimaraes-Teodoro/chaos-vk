use std::sync::Arc;

use vulkano::{command_buffer::{CopyBufferToImageInfo, CopyImageToBufferInfo}, format::Format, image::{Image, ImageCreateInfo, ImageType, ImageUsage}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use super::{buffer::VkIterBuffer, command::{submit_cmd_buf, BuilderType, VkBuilder}, vk::{MemAllocators, Vk}};

pub struct VkImage {
    pub content: Arc<Image>,
}

impl VkImage {
    pub fn sampler_host(allocators: Arc<MemAllocators>, format: Format, extent: [u32; 3]) -> Self {
        Self {
            content: Image::new(
                allocators.memory.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: format,
                    extent,
                    usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST,
                    ..Default::default()
                },
            )
            .unwrap(),
        }
    }

    pub fn depth(allocators: Arc<MemAllocators>, format: Format, extent: [u32; 3]) -> Self {
        Self {
            content: Image::new(
                allocators.memory.clone(), 
                ImageCreateInfo {
                    format: format,
                    extent,
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSFER_SRC,
                    ..Default::default()
                }, 
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        }
    }

    /// Assuming data is R8G8B8A8
    pub fn copy_buffer_to_image(
        &self, 
        vk: Arc<Vk>,
        data: &[u8],
    ) {
        let image = &self.content;
        let extent = self.content.extent();

        let mut builder = VkBuilder::new_multiple(vk.clone());

        let buffer = VkIterBuffer::transfer_src(
            vk.allocators.clone(), 
            (0..(extent[0]*extent[1]*4)).map(|i| {
                data[i as usize]
            }),
        );

        builder.0
            .copy_buffer_to_image(
                CopyBufferToImageInfo::buffer_image(buffer.content.clone(), image.clone())
            )
            .unwrap();

        let cmd_buf = builder.command_buffer();
        
        let fut = submit_cmd_buf(vk.clone(), cmd_buf);
        fut.wait(None).unwrap();
    }

    pub fn copy_image_to_buffer(
        &self, 
        vk: Arc<Vk>,
    ) -> VkIterBuffer<u8> {
        let image = &self.content;
        let extent = self.content.extent();

        let mut builder = VkBuilder::new_multiple(vk.clone());

        let buffer = VkIterBuffer::transfer_dst(
            vk.allocators.clone(), 
            (0..(extent[0]*extent[1]*4)).map(|_| {
                0u8
            }),
        );

        builder.0
            .copy_image_to_buffer(
                CopyImageToBufferInfo::image_buffer(image.clone(), buffer.content.clone())
            )
            .unwrap();

        let cmd_buf = builder.command_buffer();
        
        let fut = submit_cmd_buf(vk.clone(), cmd_buf);
        fut.wait(None).unwrap();

        buffer
    }

    pub fn submit_copy_image_to_buffer(
        &self, 
        vk: Arc<Vk>,
        builder: &mut BuilderType,
    ) -> VkIterBuffer<u8> {
        let image = &self.content;
        let extent = self.content.extent();

        let buffer = VkIterBuffer::transfer_dst(
            vk.allocators.clone(), 
            (0..(extent[0]*extent[1]*4)).map(|_| {
                0u8
            }),
        );

        builder
            .copy_image_to_buffer(
                CopyImageToBufferInfo::image_buffer(image.clone(), buffer.content.clone())
            )
            .unwrap();

        buffer
    }
    
}