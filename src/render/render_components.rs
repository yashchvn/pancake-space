use crate::render::mesh_manager::MeshID;

pub struct Renderable {
    pub mesh_id: MeshID,
}

impl Renderable {
    pub fn new(mesh_id: MeshID) -> Self {
        Self { mesh_id }
    }
}
