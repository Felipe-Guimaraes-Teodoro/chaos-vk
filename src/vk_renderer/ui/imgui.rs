use glfw::Window;
use imgui::{Context, Ui};
use vulkano::command_buffer::{CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo};

use super::super::{renderer::Renderer, command::VkBuilder, shaders::{graphics_pipeline::framebuffers, renderpass::VkSecRenderpass}};
use super::renderer::ImRenderer;

pub struct ImGui {
    last_frame: f64,
    pub renderer: Option<ImRenderer>,
    pub ctx: Context,
}

impl ImGui {
    pub fn new(window: &mut Window) -> Self {
        let mut ctx = imgui::Context::create();

        ctx.set_ini_filename(None);

        let renderer = None;

        Self {
            last_frame: window.glfw.get_time(),
            renderer, 
            ctx,
        }
    }

    pub fn setup_renderer(&mut self, renderer: &mut Renderer) {
        self.renderer = Some(ImRenderer::new(
                    &mut self.ctx,
                    renderer.vk.clone(),
                    renderer.presenter.swapchain.image_format(),
                ).unwrap());
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
        if let Some(im_renderer) = &mut self.renderer {
            let framebuffers = framebuffers(
                im_renderer.render_pass.clone(), 
                &renderer.presenter.images
            );
    
            let draw_data = self.ctx.render();
    
            /*  maybe just use a primary command buffer for imgui
                instead of relying on this
             */
            for framebuffer in &framebuffers {
                let mut builder = VkBuilder::new_secondary(
                    renderer.vk.clone(), 
                    Some(CommandBufferInheritanceInfo {
                        render_pass: Some(vulkano::command_buffer::CommandBufferInheritanceRenderPassType::BeginRenderPass(CommandBufferInheritanceRenderPassInfo {
                            subpass: im_renderer.subpass.clone(),
                            framebuffer: Some(framebuffer.clone()),
                        })),
                        ..Default::default()
                    })
                );
    
                im_renderer.draw_commands(
                    &mut builder, 
                    framebuffer.clone(), 
                    draw_data,
                    renderer.vk.clone()
                );
    
                let cmd_buf = builder.build().unwrap();
    
                let pass = VkSecRenderpass {
                    cmd_buf,
                    framebuffer: framebuffer.clone(),
                    rp: im_renderer.render_pass.clone(),
                    clear_values: vec![None],
                };
    
                renderer.sec_renderpasses.push(
                    pass,
                );
            }

        }
    }
}