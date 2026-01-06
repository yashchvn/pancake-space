use glam::{Mat4, Vec4};
use miniquad::{Bindings, BufferId};

#[repr(C)]
pub struct Instance {
    pub model_col0: Vec4,
    pub model_col1: Vec4,
    pub model_col2: Vec4,
    pub model_col3: Vec4,
    pub color: Vec4,
}

impl Instance {
    pub fn new(model: Mat4, color: Vec4) -> Self {
        Self {
            model_col0: model.x_axis,
            model_col1: model.y_axis,
            model_col2: model.z_axis,
            model_col3: model.w_axis,
            color,
        }
    }
}

pub struct MeshBatch {
    pub index_count: i32,
    pub instance_buffer: BufferId,
    pub instances: Vec<Instance>,
    pub bindings: Bindings,
}
