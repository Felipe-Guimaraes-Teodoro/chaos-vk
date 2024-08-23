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

use crate::vk_renderer::buffer::_example_operation;
use crate::vk_renderer::Vk;

use super::shaders::pipeline_shader;
use super::vertex::Vertex;

pub fn test() {
    let vk = Arc::new(Vk::new());

    _example_operation(vk.clone());

    /* example pipeline testing */

    // let buffer = VkIterBuffer::storage(vk.allocators.clone(), 0..65536u32);
    mandelbrot_image(vk);

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
    let vertices = vec![
        Vertex::from_vec(vec3(0.0, 0.0, 0.0)),
        Vertex::from_vec(vec3(0.5, 0.0, 0.0)),
        Vertex::from_vec(vec3(0.5, 0.5, 0.0)),
    ];

    let vertex_buffer = VkIterBuffer::vertex(vk.allocators.clone(), vertices);

    let framebuffer = pipeline_shader::framebuffer(vk.clone());

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
        .end_render_pass(SubpassEndInfo::default())
        .unwrap();


}