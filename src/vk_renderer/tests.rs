use crate::vk_renderer::buffer::VkIterBuffer;
use crate::vk_renderer::command::{submit_cmd_buf, VkBuilder};
use crate::vk_renderer::pipeline::VkComputePipeline;
use crate::vk_renderer::shaders::mandelbrot_shader;
use std::sync::Arc;

use glam::vec3;
use image::{ImageBuffer, Rgba};
use vulkano::command_buffer::{CopyImageToBufferInfo, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents};
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::{Pipeline, PipelineBindPoint};

use crate::vk_renderer::Vk;

use super::events::event_loop::EventLoop;
use super::geometry::fundamental::circle;
use super::graphics::mesh::Mesh;
use super::pipeline::VkGraphicsPipeline;
use super::renderer::Renderer;
use super::shaders::{fragment_shader, graphics_pipeline, vertex_shader};
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
            extent: [1024, 1024, 1],
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
        (0..1024*1024*4).map(|_| 0u8)
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
        .dispatch([1024/8, 1024/8, 1])
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
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &result[..])
        .unwrap();

    image.save("image.png").unwrap();

    println!("everything succeeded here aswell!");
}

pub fn rendering_pipeline(vk: Arc<Vk>) {
    dbg!(1024);
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
        (0..1024*1024*4).map(|_| 0u8)
    );

    let vertices = vec![
        Vertex::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0)),
        Vertex::new(vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0)),
        Vertex::new(vec3(1.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)),
    ];

    let vertex_buffer = VkIterBuffer::vertex(vk.allocators.clone(), vertices);

    let vs = vertex_shader::load(vk.device.clone()).unwrap();
    let fs = fragment_shader::load(vk.device.clone()).unwrap();

    let pipeline = VkGraphicsPipeline::new(
        vk.clone(), 
        vs.clone(),
        fs.clone(),
        VkGraphicsPipeline::default_layout(vk.clone(), vs.clone(), fs.clone()),
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

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &result[..])
        .unwrap();

    image.save("loco.png").unwrap();

    println!("graphics pipeline checks out!");
}

pub fn windowing() {
    let mut el = EventLoop::new(1200, 900);

    let mut renderer = Renderer::new(&mut el);
    renderer.presenter.window_resized = true;

    let circle = circle(16, 1.0);

    {
        let mut vertices = vec![];
    
        let mut indices = vec![];
    
        for i in 0..125000 {
            let mut vertices_i = Vec::new();

            for v in &circle.vertices {
                let mut pos = v.pos;
                let mut col = v.col;

                let w = 50;
                let d = 50;

                pos[0] += (i / (w * d)) as f32;
                pos[1] += ((i % (w * d)) / w) as f32;
                pos[2] += (i % w) as f32;

                col[0] = (i / (w * d)) as f32 / w as f32;
                col[1] = ((i % (w * d)) / w) as f32 / d as f32;
                col[2] = (i % w) as f32 / w as f32;
            
                let n_v = Vertex { pos, col, norm: [0.0, 0.0, 1.0] };
            
                vertices_i.push(n_v);
            }

            let indices_i = circle.indices.iter().map(|&idx| {
                idx + indices.len() as u32
            }).collect::<Vec<u32>>();

            vertices.extend(vertices_i);

            indices.extend(indices_i);
        }
    
        let mesh = Mesh::new(&vertices, &indices, &renderer);
        
        
        renderer.meshes.push(
            mesh
        );
    }
    

    el.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    while !el.window.should_close() {
        el.update(&mut renderer);
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos, &el);
        
        if el.is_key_down(glfw::Key::LeftAlt) {
            el.window.set_cursor_mode(glfw::CursorMode::Normal);
        } else {
            el.window.set_cursor_mode(glfw::CursorMode::Disabled);
        }
        
        renderer.update(&mut el);
    }
}

/* 
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
            presenter.window_resized = true;
        }
        Event::MainEventsCleared => {
            // renderer.update(get_circle(renderer.vk.allocators.clone(), t as usize));
            presenter.update(&renderer, renderer.vk.clone());
            presenter.recreate_swapchain = true;

            t+=0.01;
        }
        _ => (),
    });
*/