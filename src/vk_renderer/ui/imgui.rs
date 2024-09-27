use std::sync::{Arc, Mutex};

use glfw::Window;
use imgui::{Context, Ui};

use crate::renderer::Renderer;

use super::super::{presenter::Presenter, Vk};
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

    pub fn draw(&mut self, im_renderer: Arc<Mutex<ImRenderer>>, renderer: Arc<Mutex<Renderer>>) {
        let draw_data = Arc::new(self.ctx.render());
        
        let mut renderer = renderer.lock().unwrap();
        let vk = renderer.vk.clone();
        let framebuffers = renderer.presenter.framebuffers.clone();
        
        renderer.cmd_buf_callbacks.push(Box::new(move |builder| {
            im_renderer.lock().unwrap().draw_commands(
                builder,
                framebuffers[0].clone(),
                &draw_data.clone(),
                todo!(),
            );
            
        }));
    }
}