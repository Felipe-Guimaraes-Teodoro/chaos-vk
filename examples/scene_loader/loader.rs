use std::{fs::File, io::Write, sync::Arc};

use chaos_vk::graphics::{buffer::VkIterBuffer, vertex::{InstanceData, RVertex}, vk::Vk};
use glam::{quat, vec3};
use serde::{Deserialize, Serialize};

use super::{mesh::Mesh, renderer::Renderer};

#[derive(Serialize, Deserialize)]
pub struct MeshData {
    verts: Vec<[f32;3]>,
    inds: Vec<u32>,
    instances: Vec<[f32; 3]>,
    pos: [f32; 3],
    rot: [f32; 4],
    sca: [f32; 3],
    col: [f32; 3],
}

impl MeshData {
    pub fn to_mesh(&self, vk: Arc<Vk>) -> Mesh {
        let vertices: Vec<RVertex> = self.verts.iter().map(|v| {
            RVertex { pos: *v }
        }).collect();

        let instances: Vec<InstanceData> = self.instances.iter().map(|i| {
            InstanceData {ofs: *i}
        }).collect();

        let position = vec3(self.pos[0], self.pos[1], self.pos[2]);
        let rotation = quat(self.rot[0], self.rot[1], self.rot[2], self.rot[3]);
        let scale = vec3(self.sca[0], self.sca[1], self.sca[2]);
        let color = vec3(self.col[0], self.col[1], self.col[2]);

        Mesh {
            vertices: vertices.clone(),
            indices: self.inds.clone(),
            instances: instances.clone(),
            position,
            rotation,
            scale,
            color,
            
            ibo: VkIterBuffer::vertex(vk.allocators.clone(), instances),
            vbo: VkIterBuffer::vertex(vk.allocators.clone(), vertices),
            ebo: VkIterBuffer::index(vk.allocators.clone(), self.inds.clone()),
        }
    }

    pub fn from_mesh(mesh: &Mesh) -> Self {
        let verts: Vec<[f32; 3]> = mesh.vertices.iter().map(|v| v.pos).collect();
        let instances: Vec<[f32; 3]> = mesh.instances.iter().map(|i| i.ofs).collect();
        let inds = mesh.indices.clone();
        let pos = [mesh.position.x, mesh.position.y, mesh.position.z];
        let rot = [mesh.rotation.w, mesh.rotation.x, mesh.rotation.y, mesh.rotation.z];
        let sca = [mesh.scale.x, mesh.scale.y, mesh.scale.z];
        let col = [mesh.color.x, mesh.color.y, mesh.color.z];

        MeshData {
            verts,
            inds,
            instances,
            pos,
            rot,
            sca,
            col,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub meshes: Vec<MeshData>,
}

impl Scene {
    pub fn read(path: &str, renderer: &mut Renderer, vk: Arc<Vk>) -> std::io::Result<()> {
        let data = std::fs::read(path)?;
        let decoded = bincode::deserialize::<Scene>(&data)
            .expect("could not decode scene");

        let mut meshes = vec![];
        for mesh in decoded.meshes {
            let mesh = mesh.to_mesh(vk.clone());

            meshes.push(mesh);
        }

        renderer.meshes = meshes;

        Ok(())
    }

    pub fn write(path: &str, renderer: &Renderer) -> std::io::Result<()> {
        let mut scene = Scene {meshes: vec![]};

        for mesh in &renderer.meshes {
            scene.meshes.push(
                MeshData::from_mesh(mesh)
            );
        }   

        let encoded = bincode::serialize(&scene)
            .expect("could not encode scene");

        let mut file = File::create(path)?;
        file.write(&encoded)?;

        Ok(())
    }
}