use glam::{Mat4, Vec3, Vec4};
use miniquad::*;

use crate::render::{mesh_batch::Instance, mesh_manager::MeshManager, shader::*, vertex::Vertex};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, -5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 45.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 10000.0,
        }
    }

    pub fn view_projection(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let projection = Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
        projection * view
    }
}

pub struct Renderer {
    solid_pipeline: Pipeline,
    wireframe_pipeline: Pipeline,
}

impl Renderer {
    pub fn new(ctx: &mut Box<dyn RenderingBackend>) -> Self {
        let buffer_layout = BufferLayout {
            stride: std::mem::size_of::<Vertex>() as i32,
            step_func: VertexStep::PerVertex,
            ..Default::default()
        };

        let instance_buffer_layout = BufferLayout {
            stride: std::mem::size_of::<Instance>() as i32,
            step_func: VertexStep::PerInstance,
            ..Default::default()
        };

        let attributes = vec![
            // Vertex buffer (buffer index 0)
            VertexAttribute::with_buffer("in_pos", VertexFormat::Float3, 0),
            // Instance buffer (buffer index 1)
            // Mat4 takes up 4 attribute slots (4 Vec4s)
            VertexAttribute::with_buffer("model0", VertexFormat::Float4, 1),
            VertexAttribute::with_buffer("model1", VertexFormat::Float4, 1),
            VertexAttribute::with_buffer("model2", VertexFormat::Float4, 1),
            VertexAttribute::with_buffer("model3", VertexFormat::Float4, 1),
            VertexAttribute::with_buffer("in_color", VertexFormat::Float4, 1),
        ];

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: &load_shader("src/assets/shaders/vert.glsl"),
                    fragment: &load_shader("src/assets/shaders/frag.glsl"),
                },
                shader_meta(),
            )
            .unwrap();

        // Solid pipeline (fills triangles)
        let occlusion_params = PipelineParams {
            depth_test: Comparison::LessOrEqual,
            depth_write: true,
            depth_write_offset: Some((-1.0, -1.0)),
            primitive_type: PrimitiveType::Triangles,
            ..Default::default()
        };

        let solid_pipeline = ctx.new_pipeline(
            &[buffer_layout.clone(), instance_buffer_layout.clone()],
            &attributes,
            shader,
            occlusion_params,
        );

        // Wireframe pipeline (lines, no depth write)
        let wireframe_params = PipelineParams {
            depth_test: Comparison::Less,
            depth_write: true,
            primitive_type: PrimitiveType::Lines,
            ..Default::default()
        };

        let wireframe_pipeline = ctx.new_pipeline(
            &[buffer_layout, instance_buffer_layout],
            &attributes,
            shader,
            wireframe_params,
        );

        Self {
            solid_pipeline,
            wireframe_pipeline,
        }
    }

    pub fn draw(
        &mut self,
        ctx: &mut Box<dyn RenderingBackend>,
        mesh_manager: &mut MeshManager,
        view_proj: Mat4,
    ) {
        ctx.apply_pipeline(&self.solid_pipeline);
        for batch in mesh_manager.iter_batches() {
            if batch.instances.is_empty() {
                continue;
            }
            // println!("=== DRAW CALL ===");
            // println!("Instance count: {}", batch.instances.len());
            // println!("Index count: {}", batch.index_count);
            // println!("Instance buffer: {:?}", batch.instance_buffer);
            // println!("Vertex buffers: {:?}", batch.bindings.vertex_buffers);
            // println!("Index buffer: {:?}", batch.bindings.index_buffer);

            ctx.buffer_update(batch.instance_buffer, BufferSource::slice(&batch.instances));
            ctx.apply_bindings(&batch.bindings);
            ctx.apply_uniforms(UniformsSource::table(&view_proj));
            ctx.draw(0, batch.index_count, batch.instances.len() as i32);
            // println!("Draw call completed");
        }

        ctx.apply_pipeline(&self.wireframe_pipeline);
        for batch in mesh_manager.iter_batches() {
            if batch.instances.is_empty() {
                continue;
            }

            ctx.buffer_update(batch.instance_buffer, BufferSource::slice(&batch.instances));
            ctx.apply_bindings(&batch.bindings);
            ctx.apply_uniforms(UniformsSource::table(&(
                view_proj,
                Vec4::new(1.0, 1.0, 1.0, 1.0),
            )));
            ctx.draw(0, batch.index_count, batch.instances.len() as i32);
        }

        mesh_manager.clear_instance_buffer();
    }
}
