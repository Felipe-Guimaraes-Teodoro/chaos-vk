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
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

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

///  Struct to create some of vulkano's most utilized objects
pub struct Vk {
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub queue_family_index: u32,
    pub allocators: Arc<MemAllocators>,
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface>,
    pub window: Arc<Window>,
}

impl Vk {
    pub fn new(el: &EventLoop<()>) -> Arc<Self> {     
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");

        
        let required_extensions = Surface::required_extensions(el);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create instance");

        let window = Arc::new(WindowBuilder::new().build(&el).unwrap());
        let surface = Surface::from_window(instance.clone(), window.clone())
            .unwrap();

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

        Arc::new(Self {
            queue,
            physical_device,
            device,
            queue_family_index,
            allocators: Arc::new(allocators),
            instance: instance,
            surface,
            window,
        })
    }
}

pub fn swapchain(
    vk: Arc<Vk>,
) -> (Arc<vulkano::swapchain::Swapchain>, Vec<Arc<vulkano::image::Image>>) {
    let caps = vk.physical_device
        .surface_capabilities(&vk.surface, Default::default())
        .expect("failed to get surface caps");

    let size = vk.window.inner_size();
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
            present_mode: vulkano::swapchain::PresentMode::Immediate,
            image_extent: size.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap()
}
