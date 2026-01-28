use std::{collections::HashMap, fs::read_to_string};

use glam::Vec3;
use miniquad::BufferSource;
use miniquad::{Bindings, BufferType, BufferUsage, RenderingBackend};

use crate::render::mesh_batch::Instance;
use crate::render::{
    mesh_batch::MeshBatch,
    vertex::{Mesh, Vertex},
};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct MeshID(pub usize);

impl MeshID {
    pub const INVALID: Self = MeshID(0);
}

pub const MAX_INSTANCES: usize = 10_000;

pub struct MeshManager {
    pub mesh_batches: HashMap<MeshID, MeshBatch>,
    mesh_id_lookup: HashMap<String, MeshID>,
    next_mesh_id: usize,
}

impl MeshManager {
    pub fn new() -> Self {
        Self {
            mesh_batches: HashMap::new(),
            mesh_id_lookup: HashMap::new(),

            next_mesh_id: 1,
        }
    }

    pub fn register_mesh(&mut self, ctx: &mut Box<dyn RenderingBackend>, filepath: &str) -> MeshID {
        let mesh = load_obj(filepath);
        let id = MeshID(self.next_mesh_id);
        self.next_mesh_id += 1;

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&mesh.vertices),
        );
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&mesh.indices),
        );
        let instance_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<Instance>(MAX_INSTANCES),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer, instance_buffer],
            index_buffer,
            images: vec![],
        };

        let batch = MeshBatch {
            index_count: mesh.indices.len() as i32,
            instance_buffer,
            instances: vec![],
            bindings,
        };

        self.mesh_batches.insert(id, batch);
        self.mesh_id_lookup.insert(filepath.to_string(), id);

        id
    }

    pub fn submit_mesh_instance(&mut self, instance: Instance, mesh_id: MeshID) {
        if let Some(batch) = self.mesh_batches.get_mut(&mesh_id) {
            if batch.instances.len() < MAX_INSTANCES {
                // println!("Submitting instance with model: {:?}", instance.model());
                // println!("Color: {:?}", instance.color);

                batch.instances.push(instance);
            } else {
                eprintln!(
                    "Warning: Max instances ({}) reached for mesh {:?}",
                    MAX_INSTANCES, mesh_id
                );
            }
        }
    }

    pub fn clear_instance_buffer(&mut self) {
        // Clear all instance buffers for the new frame
        for batch in self.mesh_batches.values_mut() {
            batch.instances.clear();
        }
    }

    pub fn get_mesh_id(&mut self, filepath: &str) -> MeshID {
        if let Some(&id) = self.mesh_id_lookup.get(filepath) {
            return id;
        }

        return MeshID::INVALID;
    }

    pub fn iter_batches(&self) -> impl Iterator<Item = &MeshBatch> {
        self.mesh_batches.values()
    }

    pub fn iter_batches_mut(&mut self) -> impl Iterator<Item = &mut MeshBatch> {
        self.mesh_batches.values_mut()
    }
}

fn load_obj(path: &str) -> Mesh {
    let text = read_to_string(path).unwrap();

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let tag = parts.next().unwrap();

        match tag {
            "v" => {
                let x: f32 = parts.next().unwrap().parse().unwrap();
                let y: f32 = parts.next().unwrap().parse().unwrap();
                let z: f32 = parts.next().unwrap().parse().unwrap();

                vertices.push(Vertex {
                    position: Vec3::new(x, y, z),
                });
            }

            "f" => {
                let face: Vec<u32> = parts
                    .map(|p| {
                        let i: i32 = p.split('/').next().unwrap().parse().unwrap();
                        let idx = if i < 0 {
                            vertices.len() as i32 + i + 1
                        } else {
                            i
                        };
                        (idx - 1) as u32
                    })
                    .collect();

                // Fan triangulation
                for i in 1..face.len() - 1 {
                    indices.push(face[0] as u32);
                    indices.push(face[i] as u32);
                    indices.push(face[i + 1] as u32);
                }
            }

            _ => {} // ignore everything else
        }
    }

    Mesh { vertices, indices }
}
