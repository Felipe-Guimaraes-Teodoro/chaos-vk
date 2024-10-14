use chaos_vk::graphics::camera::Camera;

use super::mesh::Mesh;

pub struct Renderer {
    pub camera: Camera,

    pub meshes: Vec<Mesh>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
            meshes: vec![],
        }
    }

    pub fn update(&mut self) {
        self.camera.dt = 0.16;
        self.camera.move_according_to_input();
        self.camera.update();
    }
}