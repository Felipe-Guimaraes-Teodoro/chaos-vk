use std::sync::Arc;

use chaos_vk::{buffer::VkBuffer, command::{submit_cmd_buf, CommandBufferType, VkBuilder}, events::event_loop::EventLoop, geometry::fundamental::sphere, graphics::mesh::Mesh, pipeline::VkComputePipeline, renderer::Renderer, Vk};
use glam::vec3;
use vulkano::{buffer::BufferContents, descriptor_set::WriteDescriptorSet, pipeline::Pipeline};


mod phyisics_compute {
    vulkano_shaders::shader!{
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer Points {
                vec4 positions[64]; 
                vec4 velocities[64]; 
            } points;

            const float deltaTime = 0.1;
            const float radius = 0.5;

            void main() {
                uint idx = gl_GlobalInvocationID.x;

                if (idx >= points.positions.length()) {
                    return;
                }

                points.positions[idx] += points.velocities[idx] * deltaTime;

                float y = points.positions[idx].y;
                if (points.positions[idx].y < 0.0) {
                    points.positions[idx].y = 0.0;
                    points.velocities[idx].y = -points.velocities[idx].y * 0.5;
                    points.velocities[idx] *= 0.9;
                }

                points.velocities[idx] += vec4(0.0, -0.098, 0.0, 0.0) * deltaTime;
            }

        ",
    }
}

const MAX_BUF_LEN: usize = 64;

#[derive(BufferContents)]
#[repr(C)]
struct Point {
    positions: [[f32; 4]; MAX_BUF_LEN],
    velocities: [[f32; 4]; MAX_BUF_LEN],
}

fn main() {
    let mut el = EventLoop::new(1200, 900);
    let mut renderer = Renderer::new(&mut el);
    renderer.presenter.window_resized = true;

    let mut physics_pipeline = VkComputePipeline::new(
        renderer.vk.clone(), 
        phyisics_compute::load(renderer.vk.device.clone()).unwrap(),
    );
    let buffer = VkBuffer::storage(
        renderer.vk.allocators.clone(), 
        Point { 
            positions: (0..MAX_BUF_LEN).map(|i| { [i as f32, 10.0, 0.0, 0.0] }).collect::<Vec<_>>().try_into().unwrap(),
            velocities: (0..MAX_BUF_LEN).map(|i| { [i as f32 / MAX_BUF_LEN as f32, i as f32 / MAX_BUF_LEN as f32, i as f32 / MAX_BUF_LEN as f32, 0.0] }).collect::<Vec<_>>().try_into().unwrap(),
        },
    );
    physics_pipeline.set_descriptor_set_writes(
        [WriteDescriptorSet::buffer(0, buffer._content.clone())]
    );

    let mut positions;
    
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

        physics_pipeline.dispatch();
        let cmd_buf = get_command_buffer(renderer.vk.clone(), &mut physics_pipeline);
        let future = submit_cmd_buf(renderer.vk.clone(), cmd_buf);
        future.wait(None).unwrap();

        positions = buffer._content.read().unwrap().positions.to_vec();

        let mut i = 0;
        for position in &positions {
            if renderer.meshes.len() < positions.len() {
                let new_circle = sphere(16, 1.0);
                renderer.meshes.push(
                    Mesh::new(&new_circle.vertices, &new_circle.indices, &renderer)
                );
            }

            renderer.meshes[i].position = 
                vec3(position[0], position[1], position[2]);

            i+=1;
        }

        for mesh in &mut renderer.meshes {
            for vertex in &mut mesh.vertices {
                let mut col = vec3(mesh.position.x, mesh.position.y, mesh.position.z) * 2.0 - 1.0;
                col /= 100.0;
                vertex.col = [col.x, col.y, col.z];
            }
        }

        for mesh in &mut renderer.meshes {
            mesh.rebuild(renderer.vk.clone());  // Use the separate immutable reference
        }


        renderer.update(&mut el);
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