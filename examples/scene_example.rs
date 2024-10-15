#![allow(deprecated)]

mod scene_loader;

use std::{sync::Arc, thread::sleep_ms};

use chaos_vk::{graphics::{buffer::{VkBuffer, VkIterBuffer}, command::{CommandBufferType, VkBuilder}, presenter::Presenter, utils::{descriptor_set, instancing_pipeline, pipeline, render_pass_with_depth}, vertex::InstanceData, vk::Vk}, util::math::rand_betw};
use glam::Mat4;
use scene_loader::{geometry::sphere, loader::Scene, mesh::Mesh, renderer::Renderer, shaders::{self, vs}};
use vulkano::{command_buffer::{RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, descriptor_set::WriteDescriptorSet, pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline}, render_pass::Framebuffer};
use winit::{dpi::PhysicalSize, event::{DeviceEvent, Event, VirtualKeyCode, WindowEvent}, event_loop::EventLoop, window::{Icon, Theme}};

fn main() {
    example();
}

fn example() {
    let el = EventLoop::new();
    let vk = Vk::new(&el);
    vk.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
    vk.window.set_cursor_visible(false);
    vk.window.set_title("CHAOS-VK");
    vk.window.set_window_icon({
        let w = 8;
        let h = 8;

        Some(Icon::from_rgba(
            (0..w*h*4).map(|_| rand_betw(0, 255)).collect(), 
            w, 
            h
        ).unwrap())
    });
    vk.window.set_theme(Some(Theme::Dark));
    vk.window.set_inner_size(PhysicalSize::new(1200, 900));
    let size = vk.window.inner_size();

    let mut presenter = Presenter::new(vk.clone());
    let mut renderer = Renderer::new();
    renderer.camera.proj = Mat4::perspective_lh(
        80.0f32.to_radians(), 
        size.width as f32 / size.height as f32, 0.1, 1000.0
    );

    let vs = shaders::vs::load(vk.device.clone()).unwrap();
    let fs = shaders::fs::load(vk.device.clone()).unwrap();

    let rp = render_pass_with_depth(vk.clone(), Some(presenter.swapchain.clone()));

    let pipeline = instancing_pipeline(vk.clone(), vs, fs, rp.clone(), Viewport {
        offset: [0.0, 0.0],
        extent: size.into(),
        depth_range: 0.0..=1.0,
    });

    presenter.window_resized = true;

    let sphere = sphere(5, 1.0);
    renderer.meshes.push(Mesh::new(vk.clone(), &sphere.vertices, &sphere.indices));

    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;

    let mut frame_i = 0;

    let data = (0..250).map(|_| { InstanceData {
            ofs: [rand_betw(-100.0, 100.0), rand_betw(-10.0, 10.0), rand_betw(-100.0, 100.0)]
        }
    }).collect::<Vec<InstanceData>>();
    renderer.meshes[0].instances = data.clone();
    let instance_buffer = VkIterBuffer::vertex(vk.allocators.clone(), data);
    renderer.meshes[0].ibo = instance_buffer;

    el.run(move |event, _target, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                control_flow.set_exit();
            },
            Event::WindowEvent { event, .. } => {
                renderer.camera.input(&event);

                match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                            control_flow.set_exit();
                        }

                        if input.modifiers.ctrl() && input.virtual_keycode == Some(VirtualKeyCode::S) {
                            Scene::write("assets/scene.cf", &renderer).expect("Failed to write scene");
                        }

                        
                        if input.modifiers.ctrl() && input.virtual_keycode == Some(VirtualKeyCode::L) {
                            Scene::read("assets/scene.cf", &mut renderer, vk.clone()).expect("Failed to load scene");
                        }
                    },
                    _ => ()
                }
            }

            Event::DeviceEvent {
                event: DeviceEvent::Text { codepoint }, ..
            } => {
                dbg!(codepoint);
            }

            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta },.. } => {
                cursor_x += delta.0 as f32;
                cursor_y += delta.1 as f32;

                renderer.camera.mouse_callback(cursor_x, cursor_y);
            }

            Event::MainEventsCleared => {
                if frame_i % 6 == 0 {
                    vk.window.set_window_icon({
                        let w = 8;
                        let h = 8;
                
                        Some(Icon::from_rgba(
                            (0..w*h*4).map(|_| rand_betw(0, 255)).collect(), 
                            w, 
                            h
                        ).unwrap())
                    });
                }
                vk.window.set_theme(Some(Theme::Dark));

                renderer.update();
                presenter.cmd_bufs = get_cmd_bufs(
                    vk.clone(), 
                    &renderer,
                    presenter.framebuffers.clone(), 
                    pipeline.clone()
                );
                
                presenter.recreate(vk.clone(), rp.clone());
                presenter.present(vk.clone());

                sleep_ms(12); /* let's just assume that rendering a frame takes no time at all*/
                frame_i += 1;
            }

            Event::LoopDestroyed => {
                
            }
            _ => {}
        }
    });
}

pub fn get_cmd_bufs(
    vk: Arc<Vk>, 
    renderer: &Renderer,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
) -> Vec<CommandBufferType> {
    let mut cmd_bufs: Vec<CommandBufferType> = vec![];

    let ubo = VkBuffer::uniform(vk.allocators.clone(), vs::Camera {
        view: renderer.camera.get_view(),
        proj: renderer.camera.get_proj(),
    });

    let camera_desc_set = descriptor_set(
        vk.clone(), 
        0, 
        pipeline.clone(), 
        [WriteDescriptorSet::buffer(0, ubo.content.clone())]
    ).0;

    for framebuffer in &framebuffers.clone() {
        let mut builder = VkBuilder::new_multiple(vk.clone());

        builder.0
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.1, 0.2, 0.3, 1.0].into()), Some(1.0.into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics, 
                pipeline.layout().clone(), 
                0, 
                camera_desc_set.clone(),
            )
            .unwrap();

        for mesh in &renderer.meshes {
            mesh.build_commands(vk.clone(), &mut builder.0, pipeline.clone());
        }
        
        builder.0.end_render_pass(Default::default()).unwrap();
        
        cmd_bufs.push(
            builder.command_buffer()
        );
    }

    cmd_bufs
}