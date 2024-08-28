use crate::vk_renderer::buffer::VkIterBuffer;
use crate::vk_renderer::command::{submit_cmd_buf, VkBuilder};
use crate::vk_renderer::pipeline::VkComputePipeline;
use crate::vk_renderer::shaders::mandelbrot_shader::{self, RESOLUTION};
use std::sync::Arc;

use glam::vec3;
use image::{ImageBuffer, Rgba};
use vulkano::command_buffer::{CopyImageToBufferInfo, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::{Pipeline, PipelineBindPoint};
use vulkano::swapchain::Surface;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::vk_renderer::Vk;

use super::command::CommandBufferType;
use super::geometry::fundamental::circle;
use super::pipeline::VkGraphicsPipeline;
use super::renderer::Renderer;
use super::shaders::graphics_pipeline::{framebuffers, render_pass};
use super::shaders::{fragment_shader, graphics_pipeline, vertex_shader};
use super::swapchain;
use super::vertex::Vertex;

pub fn test() {
    //let vk = Arc::new(Vk::new(None));

    //_example_operation(vk.clone());

    /* example pipeline testing */

    // let buffer = VkIterBuffer::storage(vk.allocators.clone(), 0..65536u32);
    // mandelbrot_image(vk.clone());
    // rendering_pipeline(vk);

    windowing();

    println!("Everything succeeded!");
}

pub fn mandelbrot_image(vk: Arc<Vk>) {
    let image = Image::new(
        vk.allocators.memory.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [RESOLUTION, RESOLUTION, 1],
            usage: ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )
    .unwrap();

    let view = ImageView::new(image.clone(), ImageViewCreateInfo::from_image(&image))
        .unwrap();

    let buffer = VkIterBuffer::transfer_dst(
        vk.allocators.clone(), 
        (0..RESOLUTION*RESOLUTION*4).map(|_| 0u8)
    );

    let mut pipeline = VkComputePipeline::new(
        vk.clone(), 
        mandelbrot_shader::load(vk.device.clone()).expect("failed to create shader module"),
    );
    pipeline.set_descriptor_set_writes([WriteDescriptorSet::image_view(0, view)]);
    pipeline.dispatch();

    let mut builder = VkBuilder::new_once(vk.clone());

    builder.0
        .bind_pipeline_compute(pipeline.compute_pipeline.clone().unwrap().clone())
        .unwrap()
        .bind_descriptor_sets(
            PipelineBindPoint::Compute, 
            pipeline.compute_pipeline.unwrap().layout().clone(), 
            0, 
            pipeline.descriptor_set.unwrap()
        )
        .unwrap()
        .dispatch([RESOLUTION/8, RESOLUTION/8, 1])
        .unwrap()
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
            image.clone(),
            buffer.content.clone()
        ))
        .unwrap();

    let now = std::time::Instant::now();
    let cmd_buf = builder.0.build().unwrap();
    let future = submit_cmd_buf(vk.clone(), cmd_buf.clone());
    
    future.wait(None).unwrap();
    dbg!(now.elapsed());

    let result = buffer.content.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(RESOLUTION, RESOLUTION, &result[..])
        .unwrap();

    image.save("image.png").unwrap();

    println!("everything succeeded here aswell!");
}

pub fn rendering_pipeline(vk: Arc<Vk>) {
    
    dbg!(RESOLUTION);
    let image = Image::new(
        vk.allocators.memory.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [1024, 1024, 1],
            usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
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
        (0..RESOLUTION*RESOLUTION*4).map(|_| 0u8)
    );

    let vertices = vec![
        Vertex::from_vec(vec3(0.0, 0.0, 0.0)),
        Vertex::from_vec(vec3(1.0, 0.0, 0.0)),
        Vertex::from_vec(vec3(1.0, 1.0, 0.0)),
    ];

    let vertex_buffer = VkIterBuffer::vertex(vk.allocators.clone(), vertices);

    let pipeline = VkGraphicsPipeline::new(
        vk.clone(), 
        vertex_shader::load(vk.device.clone()).unwrap(), 
        fragment_shader::load(vk.device.clone()).unwrap(),
        None,
    );

    let framebuffer = graphics_pipeline::framebuffer(pipeline.render_pass, image.clone());

    let mut builder = VkBuilder::new_once(vk.clone());

    builder.0
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
        .bind_pipeline_graphics(pipeline.graphics_pipeline.clone())
        .unwrap()
        .bind_vertex_buffers(0, vertex_buffer.content)
        .unwrap()
        .draw(
            3, 1, 0, 0
        )
        .unwrap()
        .end_render_pass(Default::default())
        .unwrap()
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image, buffer.content.clone()))
        .unwrap();

    let cmd_buf = builder.command_buffer();

    let now = std::time::Instant::now();
    let future = submit_cmd_buf(vk, cmd_buf);
    future.wait(None).unwrap();
    dbg!(now.elapsed());

    let result = buffer.content.read().unwrap();

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(RESOLUTION, RESOLUTION, &result[..])
        .unwrap();

    image.save("loco.png").unwrap();

    println!("graphics pipeline checks out!");
}

pub fn windowing() {
    let el = EventLoop::new();

    let mut renderer = Renderer::new(&el);

    el.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            renderer.presenter.window_resized = true;
        }
        Event::MainEventsCleared => {
            renderer.update();
            renderer.presenter.present(renderer.vk.clone());
        }
        _ => (),
    });
}