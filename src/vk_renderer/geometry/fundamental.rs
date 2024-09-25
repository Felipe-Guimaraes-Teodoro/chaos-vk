#![allow(unused)]

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

        vertices.push(Vertex::new(
            vec3(f32::sin(angle), f32::cos(angle), 0.0) * radius, vec3(0., 0., 1.)
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

pub fn sphere(iterations: usize, radius: f32) -> GeometryData {
    let mut vertices = vec![];
    let pi = std::f32::consts::PI;

    for lat in 0..=iterations {
        let theta = pi * lat as f32 / iterations as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=iterations {
            let phi = 2.0 * pi * lon as f32 / iterations as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta * radius;
            let y = cos_theta * radius;
            let z = sin_phi * sin_theta * radius;

            /* DONT DELETE: for use later in vertex UV coords */
            let _s = lon as f32 / iterations as f32;
            let _t = 1.0 - (lat as f32 / iterations as f32);

            let normal = vec3(x, y, z).normalize();

            vertices.push(Vertex::new(vec3(x, y, z), normal));
        }
    }

    let mut indices: Vec<u32> = vec![];
    for lat in 0..iterations {
        for lon in 0..iterations {
            let first = lat * (iterations + 1) + lon;
            let second = first + iterations + 1;

            indices.push(first as u32);
            indices.push(second as u32);
            indices.push((first + 1) as u32);

            indices.push(second as u32);
            indices.push((second + 1) as u32);
            indices.push((first + 1) as u32);
        }
    }

    GeometryData {
        vertices,
        indices,
    }
}