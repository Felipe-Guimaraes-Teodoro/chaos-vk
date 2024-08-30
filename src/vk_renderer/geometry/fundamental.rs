use glam::vec3;

use crate::vk_renderer::vertex::Vertex;

pub struct GeometryData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn circle(iterations: usize, radius: f32) -> GeometryData {
    let mut vertices = vec![];
    
    for i in 0..iterations {
        let angle = 2.0 * std::f32::consts::PI * (i as f32 / iterations as f32);

        vertices.push(Vertex::from_vec(
            vec3(f32::sin(angle), f32::cos(angle), 0.0) * radius,
        ));
    }
    
    let mut indices = vec![];
    for i in 1..=iterations-2 {
        indices.push(0); 
        indices.push(i as u32); 
        indices.push((i % iterations + 1) as u32);
    }

    GeometryData {
        vertices,
        indices: indices
    }
}