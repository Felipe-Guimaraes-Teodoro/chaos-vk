use std::sync::Arc;

use imgui::{Context, Ui};
use renderer::ImRenderer;
use vulkano::{command_buffer::{CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo}, format::Format, image::Image};
use winit::window::Window;

use crate::graphics::{command::VkBuilder, utils::{framebuffers, VkSecRenderpass}, vk::Vk};

pub mod renderer;
pub mod shaders;

pub struct ImGui {
    _last_frame: f64,
    pub renderer: Option<ImRenderer>,
    pub ctx: Context,
}

impl ImGui {
    pub fn new() -> Self {
        let mut ctx = imgui::Context::create();

        ctx.set_ini_filename(None);

        let renderer = None;

        Self {
            _last_frame: 0.0,
            renderer, 
            ctx,
        }
    }

    pub fn setup_renderer(&mut self, vk: Arc<Vk>, format: Format) {
        self.renderer = Some(ImRenderer::new(
                    &mut self.ctx,
                    vk.clone(),
                    format,
                ));
    }

    pub fn frame(&mut self, window: &Window) -> &mut Ui {
        let io = self.ctx.io_mut();

        // let now = window.glfw.get_time();
        // TODO: FIX THIS!!!
        let delta = 0.16;
        io.delta_time = delta as f32;

        let size = window.inner_size();
        io.display_size = [size.width as f32, size.height as f32];

        self.ctx.frame()
    }

    pub fn on_mouse_move(
        &mut self, xpos: f32, ypos: f32, 
    ) {
        self.ctx.io_mut().mouse_pos = [0., 0.];
        self.ctx.io_mut().mouse_pos = [xpos, ypos];
    }
    /* 
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
    */
    pub fn on_mouse_scroll(&mut self, x: f32, y: f32) {
        self.ctx.io_mut().mouse_wheel = y;
        self.ctx.io_mut().mouse_wheel_h = x;
    }

    /// Returns a sec renderpass given the target image to render to
    pub fn get_renderpasses(&mut self, target_images: Vec<Arc<Image>>, vk: Arc<Vk>) -> Vec<VkSecRenderpass> {
        if let Some(im_renderer) = &mut self.renderer {
            let mut renderpasses = Vec::new();
            let framebuffers = framebuffers(im_renderer.render_pass.clone(), &target_images);
            let draw_data = self.ctx.render();
    
            for framebuffer in &framebuffers {
                let mut builder = VkBuilder::new_secondary(
                    vk.clone(),
                    Some(CommandBufferInheritanceInfo {
                        render_pass: Some(
                            vulkano::command_buffer::CommandBufferInheritanceRenderPassType::BeginRenderPass(
                                CommandBufferInheritanceRenderPassInfo {
                                    subpass: im_renderer.subpass.clone(),
                                    framebuffer: Some(framebuffer.clone()),
                                },
                            ),
                        ),
                        ..Default::default()
                    })
                );
    
                im_renderer.draw_commands(&mut builder, framebuffer.clone(), draw_data, vk.clone());
    
                if let Ok(cmd_buf) = builder.build() {
                    renderpasses.push(VkSecRenderpass {
                        cmd_buf,
                        framebuffer: framebuffer.clone(),
                        rp: im_renderer.render_pass.clone(),
                        clear_values: vec![None],
                    });
                }
            }
            renderpasses
        } else {
            vec![]
        }
    }
    
}