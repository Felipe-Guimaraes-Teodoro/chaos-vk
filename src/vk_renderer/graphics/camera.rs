use glam::{vec3, Mat4, Vec2, Vec3, Vec4};
use glfw::Key;
use vulkano::buffer::BufferContents;
use std::{ffi::CString, sync::Arc};

use super::super::events::event_loop::EventLoop;

use super::super::{buffer::VkBuffer, Vk};

const UP: Vec3 = Vec3::Y;
const SENSITIVITY: f32 = 0.1; // todo: make this editable


#[derive(Clone, Copy, Debug)]
pub enum ProjectionType {
    Perspective,
    Orthographic,
    Isometric,
    Oblique,
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub proj: Mat4,
    pub view: Mat4,

    projection_type: ProjectionType,

    pub pos: Vec3,
    _target: Vec3,
    direction: Vec3,
    pub right: Vec3,
    pub front: Vec3,
    pub up: Vec3,

    pub pitch: f32,
    pub yaw: f32,

    pub speed: f32,

    pub dt: f32,
    last_frame: f32,

    first_mouse: bool,
    last_x: f32,
    last_y: f32,
}

impl Camera {
    pub fn new() -> Self {
        let (pitch, yaw): (f32, f32) = (0.0, -90.0);
        let pos = vec3(0.0, 0.0, 3.0);
        let target = vec3(0.0, 0.0, -1.0);
        let mut direction = (pos - target).normalize();
        direction.x = yaw.to_radians().cos() * pitch.to_radians().cos();
        direction.y = pitch.to_radians().sin();
        direction.z = yaw.to_radians().sin() * pitch.to_radians().cos();
        
        let right = UP.cross(direction).normalize();
        let up = direction.cross(right);
        let front = direction.normalize();

        let view = Mat4::look_at_rh(pos, pos + front, up);

        Self {
            proj: Mat4::perspective_rh_gl
            (70.0f32.to_radians(), 1.0, 0.1, 100000.0),
            view,

            pos,
            _target: target,
            direction,
            right,
            front,
            up,

            speed: 1.0,

            pitch,
            yaw,

            dt: 0.0,
            last_frame: 0.0,

            projection_type: ProjectionType::Perspective,

            first_mouse: true,
            last_x: 400.0,
            last_y: 400.0,
        }
    }

    pub fn update(&mut self, y: Vec3) {
        self.pos = y;
        
        self.view = Mat4::look_at_rh(
            self.pos,
            self.pos + self.front,
            self.up,
        );
        
        // let (w, h) = el.window.get_framebuffer_size();
        let (w, h) = (100, 100);

        match self.projection_type {
            ProjectionType::Orthographic => {
                let ar = w as f32 / h as f32;
        
                self.proj = Mat4::orthographic_rh(-ar, ar, -1.0, 1.0, -100.0, 100.0);
            }

            ProjectionType::Perspective => {
                self.proj = Mat4::perspective_rh
                (70.0f32.to_radians(), w as f32 / h as f32, 0.1, 1000.0);
            }

            ProjectionType::Isometric => {
                let rotate_y = Mat4::from_rotation_y(45.0_f32.to_radians());
                let rotate_x = Mat4::from_rotation_x(35.064_f32.to_radians());
                self.proj = rotate_x * rotate_y;
            }

            ProjectionType::Oblique => {
                let scale = 0.5;
                let angle = 45.0;

                let angle_rad = f32::to_radians(angle);
                let mut mat = Mat4::IDENTITY;
                *mat.col_mut(2) = Vec4::new(scale * angle_rad.cos(), scale * angle_rad.sin(), 1.0, 0.0);

                self.proj = mat;
            }
        }
    }

    pub fn input(
        &mut self,
        el: &EventLoop, 
    ) {
        let mut speed = self.speed;
        let curr_frame = el.window.glfw.get_time() as f32;
        self.dt = curr_frame - self.last_frame;
        self.last_frame = curr_frame;

        if el.is_key_down(Key::LeftShift) {
            speed *= 20.0;
        }
        
        if el.is_key_down(Key::RightShift) {
            speed *= 20.0;
        }

        if el.is_key_down(Key::W) {
            self.pos += speed * self.dt * self.front; 
        }
        if el.is_key_down(Key::S) {
            self.pos -= speed * self.dt * self.front; 
        }
        if el.is_key_down(Key::Space) {
            self.pos -= speed * self.dt * self.up;
        }
        if el.is_key_down(Key::LeftControl) {
            self.pos += speed * self.dt * self.up;
        }
        if el.is_key_down(Key::A) {
            self.pos -= speed * self.dt * self.front.cross(self.up).normalize(); 
        }
        if el.is_key_down(Key::D) {
            self.pos += speed * self.dt * self.front.cross(self.up).normalize(); 
        }
    }

    pub fn mouse_callback(
        &mut self, 
        pos: Vec2,
        window: &glfw::Window,
    ) {
        let xpos = pos.x;
        let ypos = pos.y;
        
        if window.get_cursor_mode() != glfw::CursorMode::Disabled {
            self.first_mouse = true;
            // return 
        };
        if self.first_mouse { 
            self.last_x = xpos;
            self.last_y = ypos;
            self.first_mouse = false;
        }

        let mut xoffs = xpos - self.last_x;
        let mut yoffs = self.last_y - ypos;

        self.last_x = xpos;
        self.last_y = ypos;

        xoffs *= SENSITIVITY;
        yoffs *= SENSITIVITY;

        self.yaw += xoffs;
        self.pitch += yoffs;

        self.direction.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.direction.y = self.pitch.to_radians().sin();
        self.direction.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        self.front = self.direction.normalize();

        self.right = Vec3::Y.cross(self.front).normalize();
        self.up = self.front.cross(self.right).normalize();
    }

    // RENDERING //
    pub fn get_view(&self) -> [[f32;4];4] {
        self.view.to_cols_array_2d()
    }

    pub fn get_proj(&self) -> [[f32; 4]; 4] {
        self.proj.to_cols_array_2d()
    }

    pub fn set_projection(
        &mut self, 
        projection_type: ProjectionType,
    ) {
        match projection_type {
            ProjectionType::Perspective => {
                self.projection_type = ProjectionType::Perspective;
            },
            ProjectionType::Orthographic => {
                self.projection_type = ProjectionType::Orthographic;
            },
            ProjectionType::Isometric => {
                self.projection_type = ProjectionType::Isometric;
            },
            ProjectionType::Oblique => {
                self.projection_type = ProjectionType::Oblique;
            }
        }
    }
 
 }