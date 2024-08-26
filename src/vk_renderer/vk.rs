use std::sync::Arc;

use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::image::ImageUsage;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};
use winit::event_loop::EventLoop;
use winit::window::Window;

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

/*
    STILL NEEDS BETTER INITIALIZING: CHECK WHETHER OR NOT
    DEVICE SUPPORTS SWAPCHAIN
*/
pub struct Vk {
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub queue_family_index: u32,
    pub allocators: Arc<MemAllocators>,
    pub instance: Arc<Instance>,
}

impl Vk {
    pub fn new(el: Option<&EventLoop<()>>) -> Self {     
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let instance = Instance::new(
            library, 
            InstanceCreateInfo {
                enabled_extensions: match el {
                    Some(el) => Surface::required_extensions(el),
                    None => Default::default(),
                },
                ..Default::default()
            }
        )
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
            .position(|(_, q)| q.queue_flags.contains(QueueFlags::GRAPHICS))
            .expect("couldn't find a graphical queue family") as u32;
            
        let (device, mut queues) = Device::new(
                physical_device.clone(),
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
            physical_device,
            device,
            queue_family_index,
            allocators: Arc::new(allocators),
            instance: instance,
        }
    }
}

pub fn swapchain(
    vk: Arc<Vk>, 
    surface: Arc<Surface>, 
    window: Arc<Window>
) -> (Arc<vulkano::swapchain::Swapchain>, Vec<Arc<vulkano::image::Image>>) {
    let caps = vk.physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface caps");

    let dims = window.inner_size();
    let composite_alpha = caps.supported_composite_alpha
        .into_iter()
        .next()
        .unwrap();
    let image_format = vk.physical_device
        .surface_formats(&surface, Default::default())
        .unwrap()[0]
        .0;

    Swapchain::new(
        vk.device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format,
            image_extent: dims.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap()
}