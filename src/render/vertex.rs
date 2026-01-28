use glam::Vec3;

#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
