use chaos_vk::graphics::{camera::Camera, mesh::mesh::Mesh};
use glam::Vec3;

use crate::util::math::SecondOrderDynamics;

pub struct Renderer {
    pub camera: Camera,
    pub cam_sod: SecondOrderDynamics<Vec3>,

    pub meshes: Vec<Mesh>,
}

impl Renderer {
    pub fn new() -> Self {
        let mut camera = Camera::new();
        camera.speed = 8.0;
        Self {
            camera,
            cam_sod: SecondOrderDynamics::new(3.5, 0.8, 0.0, camera.pos),
            meshes: vec![],
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.camera.dt = dt;
        self.camera.set_goal_according_to_input();
        let y = self.cam_sod.update(self.camera.dt, self.camera.goal);
        self.camera.update(y);
    }
}