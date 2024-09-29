use std::sync::Arc;

use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::image::ImageUsage;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};
use winapi::um::libloaderapi::GetModuleHandleW;

use super::events::event_loop::EventLoop;

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
                    StandardCommandBufferAllocatorCreateInfo {
                        primary_buffer_count: 1,
                        secondary_buffer_count: 1,
                        ..Default::default()
                    }
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
    pub surface: Arc<Surface>,
}

impl Vk {
    /* todo: create surface based on HasRawWindowHandle trait for cross platform compat */
    pub fn new(el: &mut EventLoop) -> Self {     
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");

        
        let required_extensions = el.glfw.get_required_instance_extensions().unwrap();
        let instance = Instance::new(
            library, 
            InstanceCreateInfo {
                enabled_extensions: required_extensions.iter().map(|string| string.as_str()).collect(),
                ..Default::default()
            }
        )
        .expect("failed to create instance");

        let hwnd = el.window.get_win32_window();
        let hinstance = unsafe { GetModuleHandleW(core::ptr::null()) as *const _ };

        let surface = unsafe {
            Surface::from_win32(instance.clone(), hinstance, hwnd, None)
                .expect("Failed to create Vulkan surface")
        };

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        /* properly select a physical device */
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,

                _ => 4,
            })
            .expect("no device available");

    
        // for family in physical_device.queue_family_properties() {
        //     println!("Found a queue family with {:?} queue(s)", family.queue_count);
        // }
            
        let (device, mut queues) = Device::new(
                physical_device.clone(),
                DeviceCreateInfo {
                    queue_create_infos: vec![QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }],
                    enabled_extensions: device_extensions,
                    ..Default::default()
                },
            )
            .expect("failed to create device");

        let allocators = MemAllocators::new(device.clone());
    
        let queue = queues.next().unwrap();

        Self {
            queue,
            physical_device,
            device,
            queue_family_index,
            allocators: Arc::new(allocators),
            instance: instance,
            surface,
        }
    }
}

pub fn swapchain(
    vk: Arc<Vk>,
    el: &EventLoop,
) -> (Arc<vulkano::swapchain::Swapchain>, Vec<Arc<vulkano::image::Image>>) {
    let caps = vk.physical_device
        .surface_capabilities(&vk.surface, Default::default())
        .expect("failed to get surface caps");

    let (w, h) = el.window.get_size();
    let composite_alpha = caps.supported_composite_alpha
        .into_iter()
        .next()
        .unwrap();
    let image_format = vk.physical_device
        .surface_formats(&vk.surface, Default::default())
        .unwrap()[0]
        .0;

    Swapchain::new(
        vk.device.clone(),
        vk.surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format,
            present_mode: vulkano::swapchain::PresentMode::Fifo,
            image_extent: [w as u32, h as u32],
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap()
}
