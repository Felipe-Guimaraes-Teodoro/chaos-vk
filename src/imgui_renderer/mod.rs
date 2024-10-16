/* TODO: rewrite this */

use std::sync::Arc;

use imgui::{Context, Ui};
use renderer::ImRenderer;
use vulkano::{command_buffer::{CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo}, image::Image};
use winit::{event::{ElementState, MouseButton}, window::Window};

use crate::graphics::{command::VkBuilder, presenter::Presenter, utils::{framebuffers, VkSecRenderpass}, vk::Vk};

pub mod renderer;
pub mod shaders;

pub struct ImGui {
    _last_frame: f64,
    pub renderer: ImRenderer,
    pub ctx: Context,
}

impl ImGui {
    pub fn new(vk: Arc<Vk>,  presenter: &Presenter) -> Self {
        let mut ctx = imgui::Context::create();

        ctx.set_ini_filename(None);

        let renderer = ImRenderer::new(
            &mut ctx,
            vk.clone(),
            presenter.swapchain.image_format(),
        );

        Self {
            _last_frame: 0.0,
            renderer, 
            ctx,
        }
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
    
    pub fn on_mouse_click(
        &mut self, button: MouseButton, state: ElementState,
    ) {
        let is_pressed = if state == ElementState::Pressed {true} else {false};

        match button {
            MouseButton::Left => { 
                self.ctx.io_mut().mouse_down[0] = is_pressed;
            },
            MouseButton::Right => {
                self.ctx.io_mut().mouse_down[1] = is_pressed;
            },
            MouseButton::Middle => {
                self.ctx.io_mut().mouse_down[2] = is_pressed;
            },
            _ => {},
        }
    }
    
    pub fn on_mouse_scroll(&mut self, x: f32, y: f32) {
        self.ctx.io_mut().mouse_wheel += y;
        self.ctx.io_mut().mouse_wheel_h += x;
    }

    /// Returns a sec renderpass given the target image to render to
    pub fn get_renderpasses(&mut self, target_images: Vec<Arc<Image>>, vk: Arc<Vk>) -> Vec<VkSecRenderpass> {
        let mut renderpasses = Vec::new();
        let framebuffers = framebuffers(self.renderer.render_pass.clone(), &target_images);
        let draw_data = self.ctx.render();

        for framebuffer in &framebuffers {
            let mut builder = VkBuilder::new_secondary(
                vk.clone(),
                Some(CommandBufferInheritanceInfo {
                    render_pass: Some(
                        vulkano::command_buffer::CommandBufferInheritanceRenderPassType::BeginRenderPass(
                            CommandBufferInheritanceRenderPassInfo {
                                subpass: self.renderer.subpass.clone(),
                                framebuffer: Some(framebuffer.clone()),
                            },
                        ),
                    ),
                    ..Default::default()
                })
            );

            self.renderer.draw_commands(&mut builder, framebuffer.clone(), draw_data, vk.clone());

            if let Ok(cmd_buf) = builder.build() {
                renderpasses.push(VkSecRenderpass {
                    cmd_buf,
                    framebuffer: framebuffer.clone(),
                    rp: self.renderer.render_pass.clone(),
                    clear_values: vec![None],
                });
            }
        }
        renderpasses
    }
    
}