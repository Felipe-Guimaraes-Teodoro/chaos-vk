use std::sync::{Arc, Mutex};

use glfw::Window;
use imgui::{Context, DrawData, Ui};
use vulkano::command_buffer::{CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo};
use vulkano::render_pass::Framebuffer;

use super::super::{presenter::Presenter, Vk, renderer::Renderer, command::VkBuilder, shaders::{graphics_pipeline::framebuffers, renderpass::VkSecRenderpass}};
use super::renderer::{self, ImRenderer};

pub struct ImGui {
    last_frame: f64,
    pub renderer: ImRenderer,
    pub ctx: Context,
}

impl ImGui {
    pub fn new(window: &mut Window, presenter: &Presenter, vk: Arc<Vk>) -> Self {
        let mut ctx = imgui::Context::create();

        ctx.set_ini_filename(None);

        let renderer = ImRenderer::new(
            &mut ctx,
            vk.clone(),
            presenter.swapchain.image_format(),
        ).unwrap();

        Self {
            last_frame: window.glfw.get_time(),
            renderer, 
            ctx,
        }
    }

    pub fn frame(&mut self, window: &mut Window) -> &mut Ui {
        let io = self.ctx.io_mut();

        let now = window.glfw.get_time();
        let delta = now - self.last_frame;
        self.last_frame = now;
        io.delta_time = delta as f32;

        let (w, h) = window.get_size();
        io.display_size = [w as f32, h as f32];

        self.ctx.frame()
    }

    pub fn on_mouse_move(
        &mut self, xpos: f32, ypos: f32, 
    ) {
        self.ctx.io_mut().mouse_pos = [0., 0.];
        self.ctx.io_mut().mouse_pos = [xpos, ypos];
    }
    pub fn on_mouse_click(
        &mut self, button: glfw::MouseButton, action: glfw::Action,
    ) {
        let is_pressed = if action == glfw::Action::Press {true} else {false};

        match button {
            glfw::MouseButton::Button1 => { 
                self.ctx.io_mut().mouse_down[0] = is_pressed;
            },
            glfw::MouseButton::Button2 => {
                self.ctx.io_mut().mouse_down[1] = is_pressed;
            },
            glfw::MouseButton::Button3 => {
                self.ctx.io_mut().mouse_down[2] = is_pressed;
            },
            glfw::MouseButton::Button4 => {
                self.ctx.io_mut().mouse_down[3] = is_pressed;
            },
            glfw::MouseButton::Button5 => {
                self.ctx.io_mut().mouse_down[4] = is_pressed;
            },
            _ => {},
        }
    }

    pub fn on_mouse_scroll(&mut self, x: f32, y: f32) {
        self.ctx.io_mut().mouse_wheel = y;
        self.ctx.io_mut().mouse_wheel_h = x;
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        let framebuffers = framebuffers(
            renderer.vk.clone(), 
            self.renderer.render_pass.clone(), 
            &renderer.presenter.images
        );

        let draw_data = self.ctx.render();

        for framebuffer in framebuffers {
            let mut builder = VkBuilder::new_secondary(
                renderer.vk.clone(), 
                Some(CommandBufferInheritanceInfo {
                    render_pass: Some(vulkano::command_buffer::CommandBufferInheritanceRenderPassType::BeginRenderPass(CommandBufferInheritanceRenderPassInfo {
                        subpass: self.renderer.subpass.clone(),
                        framebuffer: Some(framebuffer.clone()),
                    })),
                    ..Default::default()
                })
            );

            self.renderer.draw_commands(
                &mut builder, 
                framebuffer.clone(), 
                draw_data,
                renderer.vk.clone()
            );

            let cmd_buf = builder.build().unwrap();

            let pass = VkSecRenderpass {
                cmd_buf,
                framebuffer,
                rp: self.renderer.render_pass.clone(),
            };

            renderer.sec_renderpasses.push(
                pass,
            );
        }
    }
}