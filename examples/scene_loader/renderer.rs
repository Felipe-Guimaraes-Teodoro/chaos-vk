use chaos_vk::{graphics::camera::Camera, util::math::SecondOrderDynamics};
use glam::Vec3;

use super::mesh::Mesh;

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
            cam_sod: SecondOrderDynamics::new(2.75, 0.75, 0.0, camera.pos),
            meshes: vec![],
        }
    }

    pub fn update(&mut self) {
        self.camera.dt = 0.016;
        self.camera.set_goal_according_to_input();
        let y = self.cam_sod.update(self.camera.dt, self.camera.goal);
        self.camera.update(y);
    }
}