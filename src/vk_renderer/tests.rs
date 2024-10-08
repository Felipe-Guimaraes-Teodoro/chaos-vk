use crate::vk_renderer::buffer::VkIterBuffer;
use crate::vk_renderer::command::{submit_cmd_buf, VkBuilder};
use crate::vk_renderer::pipeline::VkComputePipeline;
use crate::vk_renderer::shaders::mandelbrot_shader;
use std::sync::Arc;

use glam::vec3;
use image::{ImageBuffer, Rgba};
use imgui::{ImColor32, TextureId};
use vulkano::command_buffer::{CopyImageToBufferInfo, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents};
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::{Pipeline, PipelineBindPoint};

use super::events::event_loop::EventLoop;
use super::geometry::fundamental::sphere;
use super::graphics::mesh::Mesh;
use super::pipeline::{PipelineHandle, VkGraphicsPipeline};
use super::renderer::Renderer;
use super::shaders::{fragment_shader, graphics_pipeline, vertex_shader};
use super::vertex::Vertex;
use super::Vk;

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

    el.ui.setup_renderer(&mut renderer);
    
    renderer.presenter.window_resized = true;

    let sphere = sphere(5, 1.0);

    let mut vertices = vec![];
    let mut indices = vec![];
    let mut normals = vec![];
    
    let w = 40;
    let d = 40;

    {
        for i in 0..64000 {
            let mut vertices_i = vec![];
        
            for v in &sphere.vertices {
                let mut pos = v.pos;
                let mut col = v.col;
                let norm = v.norm;
        
                pos[0] += (i / (w * d)) as f32;
                pos[1] += ((i % (w * d)) / w) as f32;
                pos[2] += (i % w) as f32;
        
                col[0] = (i / (w * d)) as f32 / w as f32;
                col[1] = ((i % (w * d)) / w) as f32 / d as f32;
                col[2] = (i % w) as f32 / w as f32;
        
                vertices_i.push(Vertex { pos, col, norm });
                normals.push(norm);
            }
        
            let start_index = indices.len() as u32;
            let indices_i: Vec<u32> = sphere.indices.iter().map(|&idx| idx + start_index).collect();
        
            vertices.extend(vertices_i);
            indices.extend(indices_i);
        }

        let mesh = Mesh::new(&vertices, &indices, &renderer);
        
        renderer.meshes.insert(
            0,
            mesh
        );
    }

    el.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    let mut elapsed = 0.0;
    let (mut y, mut x) = (0.0, 0.0);

    let dst = VkIterBuffer::transfer_src_dst(
        renderer.vk.allocators.clone(), 
        (0..1200*900*4).map(|_| 0u16),
    );
    
    while !el.window.should_close() {
        el.update(&mut renderer);
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos, &el);
        
        let ui = el.ui.frame(&mut el.window);

        y += el.event_handler.scroll.y * 20.0;
        x += el.event_handler.scroll.x * 20.0;

        {
            let dl = ui.get_foreground_draw_list();
            dl.add_rect([20.0, 20.0], [160.0, 40.0], ImColor32::BLACK)
                .rounding(4.5)
                .filled(true)
                .build();

            dl.add_text([24.0, 24.0], ImColor32::WHITE, format!("f: {:.1} | e: {:.1}", 1.0/(elapsed/1000.0), elapsed));
            
            dl.add_circle([10.0, 30.0], 5.0, ImColor32::from_rgb(255, 0, 0))
                .filled(true)
                .build();
            
            let depth_image = renderer.presenter.framebuffers[0].attachments()[1].image();

            let mut builder = VkBuilder::new_multiple(renderer.vk.clone());

            let cmd_buf = {
                builder.0
                    .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                            depth_image.clone(), 
                            dst.content.clone()
                        )
                    )
                    .unwrap();

                builder.command_buffer()
            };

            submit_cmd_buf(renderer.vk.clone(), cmd_buf)
                .wait(None).unwrap();

            dl.add_text(
                [4.0 + x, 64.0 + y], 
                ImColor32::WHITE, 
                format!(
                    "{:#?}", 
                    depth_image
                )
            );
        }

        el.ui.draw(&mut renderer);
        
        let now = std::time::Instant::now();
        renderer.draw();
        elapsed = now.elapsed().as_secs_f32() * 1000.0;
        renderer.presenter.recreate(renderer.vk.clone(), &el, &mut renderer.pipelines);


        if el.event_handler.key_just_pressed(glfw::Key::F) {
            let result = dst.content.read().unwrap();
            let mut u8_buffer: Vec<u8> = Vec::with_capacity(1200*900 * 4);

            for &depth_value in result.iter() {
                let normalized_value = ((depth_value as f32 / 65535.0) * 255.0) as u8;
                u8_buffer.extend_from_slice(&[normalized_value, normalized_value, normalized_value, 255]);
            }

            let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1200, 900, &u8_buffer[..])
                .unwrap();

            image.save("depth.png").unwrap();
        }

        if el.is_key_down(glfw::Key::LeftAlt) {
            el.window.set_cursor_mode(glfw::CursorMode::Normal);
        } else {
            el.window.set_cursor_mode(glfw::CursorMode::Disabled);
        }
    }
}
