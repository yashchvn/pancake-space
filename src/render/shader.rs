use ::std::fs;
use miniquad::*;

pub fn load_shader(path: &str) -> String {
    fs::read_to_string(path).expect(&format!("Failed to read shader: {}", path))
}

pub fn starfield_shader_meta() -> ShaderMeta {
    ShaderMeta {
        uniforms: UniformBlockLayout {
            uniforms: vec![UniformDesc::new("view_proj", UniformType::Mat4)],
        },
        images: vec![],
    }
}

pub fn geometry_shader_meta() -> ShaderMeta {
    ShaderMeta {
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("view_proj", UniformType::Mat4),
                UniformDesc::new("color_override", UniformType::Float4),
            ],
        },
        images: vec![],
    }
}
