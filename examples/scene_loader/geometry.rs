#![allow(unused)]

use chaos_vk::graphics::vertex::RVertex;
use glam::{vec2, vec3, Vec2};

pub struct GeometryData {
    pub vertices: Vec<RVertex>,
    pub indices: Vec<u32>,
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

            let s = lon as f32 / iterations as f32;
            let t = 1.0 - (lat as f32 / iterations as f32);

            let normal = vec3(x, y, z).normalize();

            vertices.push(RVertex {
                pos: [x, y, z],
            });
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