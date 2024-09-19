use std::sync::Arc;

use chaos_vk::{buffer::VkBuffer, command::{submit_cmd_buf, CommandBufferType, VkBuilder}, events::event_loop::EventLoop, geometry::fundamental::circle, graphics::mesh::Mesh, pipeline::VkComputePipeline, presenter::Presenter, renderer::Renderer, Vk};
use vulkano::{buffer::BufferContents, descriptor_set::WriteDescriptorSet, pipeline::Pipeline};


mod phyisics_compute {
    vulkano_shaders::shader!{
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 32, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer Points {
                vec3 positions[32];
                vec3 velocities[32];
            } points;

            void main() {
                uint idx = gl_GlobalInvocationID.x;

                points.positions[idx] += points.velocities[idx] * 0.01;
                // points.velocities[idx] += vec3(0.0, -0.98, 0.0); 
            }
        ",
    }
}

const MAX_BUF_LEN: usize = 32;

#[derive(BufferContents)]
#[repr(C)]
struct Point {
    positions: [[f32; 3]; MAX_BUF_LEN],
    velocities: [[f32; 3]; MAX_BUF_LEN],
}

fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new(&mut el);

    let circle = circle(16, 1.0);

    renderer.meshes.push(Mesh::new(&circle.vertices, &circle.indices, &renderer));

    let mut physics_pipeline = VkComputePipeline::new(
        renderer.vk.clone(), 
        phyisics_compute::load(renderer.vk.device.clone()).unwrap(),
    );

    let buffer = VkBuffer::storage(
        renderer.vk.allocators.clone(), 
        Point { 
            positions: (0..MAX_BUF_LEN).map(|i| { [i as f32, 0.0, 0.0] }).collect::<Vec<_>>().try_into().unwrap(),
            velocities: [[0.0, -9.0, 0.0]; MAX_BUF_LEN],
        },
    );

    physics_pipeline.set_descriptor_set_writes(
        [WriteDescriptorSet::buffer(0, buffer._content.clone())]
    );
    
    while !el.window.should_close() {
        physics_pipeline.dispatch();

        let cmd_buf = get_command_buffer(renderer.vk.clone(), &mut physics_pipeline);

        let future = submit_cmd_buf(renderer.vk.clone(), cmd_buf);
        future.wait(None).unwrap();

        dbg!(buffer._content.read().unwrap().positions);
    }
}

fn get_command_buffer(
    vk: Arc<Vk>,
    pipeline: &mut VkComputePipeline,
) -> CommandBufferType {
    let mut builder  = VkBuilder::new_multiple(vk.clone());
    let cmd_buf = {
        builder.0
            .bind_pipeline_compute(pipeline.compute_pipeline.clone().unwrap())
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Compute, 
                pipeline.compute_pipeline.clone().unwrap().layout().clone(), 
                0, 
                pipeline.descriptor_set.clone().unwrap()
            )
            .unwrap()
            .dispatch([MAX_BUF_LEN as u32, 1, 1])
            .unwrap();

        builder.0
            .build()
            .unwrap()
    };

    cmd_buf
}