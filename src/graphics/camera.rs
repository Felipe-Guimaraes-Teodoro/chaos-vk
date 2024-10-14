use glam::{vec3, Mat4, Vec3};

const SPEED: f32 = 2.0;
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

    first_mouse: bool,
    last_x: f32,
    last_y: f32,
    
    keymap: [bool; 6],
}

impl Camera {
    pub fn new() -> Self {
        let (pitch, yaw): (f32, f32) = (0.0, -90.0);
        let pos = vec3(0.0, 0.0, -10.0);
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
            proj: Mat4::perspective_lh
            (70.0f32.to_radians(), 1.0, 0.1, 1000.0),
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

            first_mouse: true,
            last_x: 400.0,
            last_y: 400.0,

            keymap: [false; 6],
        }
    }

    pub fn update(&mut self) {
        self.view = Mat4::look_at_rh(
            self.pos,
            self.pos + self.front,
            -self.up,
        );
    }

    pub fn input(
        &mut self,
        event: &winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                let action = match input.state {
                    winit::event::ElementState::Pressed => true,
                    winit::event::ElementState::Released => false,
                };

                match input.virtual_keycode {
                    Some(winit::event::VirtualKeyCode::W) => {
                        self.keymap[0] = action;
                    },
                    Some(winit::event::VirtualKeyCode::A) => {
                        self.keymap[1] = action;
                    },
                    Some(winit::event::VirtualKeyCode::S) => {
                        self.keymap[2] = action;
                    },
                    Some(winit::event::VirtualKeyCode::D) => {
                        self.keymap[3] = action;
                    },
                    Some(winit::event::VirtualKeyCode::Space) => {
                        self.keymap[4] = action;
                    },
                    Some(winit::event::VirtualKeyCode::C) => {
                        self.keymap[5] = action;
                    },
                    _ => ()
                }
            }

            _ => (),
        }
    }

    pub fn mouse_callback(
        &mut self, 
        xpos: f32, 
        ypos: f32,
    ) {
        if self.first_mouse { 
            self.last_x = xpos;
            self.last_y = ypos;
            self.first_mouse = false;
        }

        let mut xoffs = xpos - self.last_x;
        let mut yoffs = -(self.last_y - ypos);

        self.last_x = xpos;
        self.last_y = ypos;

        xoffs *= SENSITIVITY;
        yoffs *= SENSITIVITY;

        self.yaw += xoffs;
        self.pitch += yoffs;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        } 
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.direction.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.direction.y = self.pitch.to_radians().sin();
        self.direction.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        self.front = Vec3::normalize(self.direction);
    }

    pub fn move_according_to_input(&mut self) {
        if self.keymap[0] {
            self.pos -= SPEED * self.dt * self.front;
        }
        if self.keymap[1] {
            self.pos += SPEED * self.dt * Vec3::cross(self.front, self.up);
        }
        if self.keymap[2] {
            self.pos += SPEED * self.dt * self.front;
        }
        if self.keymap[3] {
            self.pos -= SPEED * self.dt * Vec3::cross(self.front, self.up);
        }
        if self.keymap[4] {
            self.pos += SPEED * self.dt * self.up;
        }
        if self.keymap[5] {
            self.pos -= SPEED * self.dt * self.up;
        }
    }

    // RENDERING //
    pub fn get_view(&self) -> [[f32;4];4] {
        self.view.to_cols_array_2d()
    }

    pub fn get_proj(&self) -> [[f32; 4]; 4] {
        self.proj.to_cols_array_2d()
    }
 
 }