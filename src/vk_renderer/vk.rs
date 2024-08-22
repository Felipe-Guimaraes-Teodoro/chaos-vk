use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{ClearColorImageInfo, CopyImageToBufferInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::format::{ClearColorValue, Format};
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};

use super::buffer::{VkIterBuffer, _example_operation};
use super::command::{submit_cmd_buf, VkBuilder};
use super::pipeline::VkComputePipeline;
pub struct MemAllocators {
    pub memory: Arc<StandardMemoryAllocator>,
    pub command: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set: Arc<StandardDescriptorSetAllocator>,
}

impl MemAllocators {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            memory: Arc::new(
                StandardMemoryAllocator::new_default(device.clone())
            ),
            command: Arc::new(StandardCommandBufferAllocator::new(
                    device.clone(), 
                    StandardCommandBufferAllocatorCreateInfo::default()
                )
            ),
            descriptor_set: Arc::new(StandardDescriptorSetAllocator::new(
                    device.clone(), 
                    Default::default()
                )
            ),
        }
    }
}

/*
    Struct to create vk's most utilized objects
*/
pub struct Vk {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub queue_family_index: u32,
    pub allocators: Arc<MemAllocators>,
}

impl Vk {
    pub fn new() -> Self {
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let instance = Instance::new(library, InstanceCreateInfo::default())
            .expect("failed to create instance");
    
        let physical_device = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .next()
            .expect("no devices available");
    
        // for family in physical_device.queue_family_properties() {
        //     println!("Found a queue family with {:?} queue(s)", family.queue_count);
        // }
    
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)| {
                queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
            })
            .expect("couldn't find a graphical queue family") as u32;
            
        let (device, mut queues) = Device::new(
                physical_device,
                DeviceCreateInfo {
                    queue_create_infos: vec![QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )
            .expect("failed to create device");
    
        let queue = queues.next().unwrap();
    
        let allocators = MemAllocators::new(device.clone());

        Self {
            queue,
            device: device,
            queue_family_index,
            allocators: Arc::new(allocators),
        }
    }
}

pub fn test() {
    let vk = Arc::new(Vk::new());

    _example_operation(vk.clone());

    /* example pipeline testing */

    let buffer = VkIterBuffer::storage(vk.allocators.clone(), 0..65536u32);
    let mut pipeline = VkComputePipeline::new(vk.clone());
    pipeline.set_descriptor_set_writes([WriteDescriptorSet::buffer(0, buffer.content.clone())]);
    pipeline.dispatch();

    let content = buffer.content.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }

    println!("Everything succeeded!");
}

pub fn mandelbrot_image() {
    let vk = Arc::new(Vk::new());

    let image = Image::new(
        vk.allocators.memory.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [1024, 1024, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )
    .unwrap();

    let buffer = VkIterBuffer::transfer_dst(
        vk.allocators.clone(), 
        (0..1024*1024*4).map(|_| 0u8)
    );

    let mut builder = VkBuilder::new_once(vk.clone());

    builder.0
        .clear_color_image(ClearColorImageInfo {
            clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
            ..ClearColorImageInfo::image(image.clone())
        })
        .unwrap()
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
            image.clone(),
            buffer.content.clone()
        ))
        .unwrap();
    let cmd_buf = builder.0.build().unwrap();
    let future = submit_cmd_buf(vk.clone(), cmd_buf.clone());

    future.wait(None).unwrap();

    let result = buffer.content.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &result[..])
        .unwrap();

    image.save("image.png").unwrap();

    println!("everything succeeded here aswell!");
}