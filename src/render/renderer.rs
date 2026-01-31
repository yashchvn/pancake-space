use glam::{Mat4, Vec3, Vec4};
use miniquad::*;
use rand::Rng;

use crate::render::{mesh_batch::Instance, mesh_manager::MeshManager, shader::*, vertex::Vertex};

pub struct Renderer {
    solid_pipeline: Pipeline,
    wireframe_pipeline: Pipeline,
    starfield_pipeline: Pipeline,
    starfield_bindings: Bindings,
}

impl Renderer {
    pub fn new(ctx: &mut Box<dyn RenderingBackend>) -> Self {
        let geom_buffer_layout = BufferLayout {
            stride: std::mem::size_of::<Vertex>() as i32,
            step_func: VertexStep::PerVertex,
            ..Default::default()
        };

        let geom_instance_buffer_layout = BufferLayout {
            stride: std::mem::size_of::<Instance>() as i32,
            step_func: VertexStep::PerInstance,
            ..Default::default()
        };

        let geom_pipeline_attributes = vec![
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

        let geom_shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: &load_shader("src/assets/shaders/geom_vert.glsl"),
                    fragment: &load_shader("src/assets/shaders/geom_frag.glsl"),
                },
                geometry_shader_meta(),
            )
            .unwrap();

        // Solid pipeline (fills triangles)
        let geom_occlusion_params = PipelineParams {
            depth_test: Comparison::Less,
            depth_write: true,
            // depth_write_offset: Some((-1.0, -1.0)),
            primitive_type: PrimitiveType::Triangles,
            ..Default::default()
        };

        let solid_pipeline = ctx.new_pipeline(
            &[
                geom_buffer_layout.clone(),
                geom_instance_buffer_layout.clone(),
            ],
            &geom_pipeline_attributes,
            geom_shader,
            geom_occlusion_params,
        );

        // Wireframe pipeline (lines, no depth write)
        let geom_wireframe_params = PipelineParams {
            depth_test: Comparison::LessOrEqual,
            // depth_write_offset: Some((1.0, 1.0)),
            depth_write: true,
            primitive_type: PrimitiveType::Lines,
            ..Default::default()
        };

        let wireframe_pipeline = ctx.new_pipeline(
            &[geom_buffer_layout, geom_instance_buffer_layout],
            &geom_pipeline_attributes,
            geom_shader,
            geom_wireframe_params,
        );

        // Starfield pipeline setup
        let starfield_pipeline_params = PipelineParams {
            depth_test: Comparison::Never,
            depth_write: false,
            primitive_type: PrimitiveType::Points,
            ..Default::default()
        };

        let starfield_shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: &load_shader("src/assets/shaders/starfield_vert.glsl"),
                    fragment: &load_shader("src/assets/shaders/starfield_frag.glsl"),
                },
                starfield_shader_meta(),
            )
            .unwrap();

        let mut rng = rand::rng();
        let starfield_vertices: Vec<Vec3> = (0..2000)
            .map(|_| {
                Vec3::new(
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                )
            })
            .collect();

        let starfield_indices: Vec<u16> = (0..starfield_vertices.len() as u16).collect();

        let starfield_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&starfield_vertices),
        );

        let starfield_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&starfield_indices),
        );

        let starfield_buffer_layout = BufferLayout {
            stride: std::mem::size_of::<Vec3>() as i32,
            step_func: VertexStep::PerVertex,
            ..Default::default()
        };

        let starfield_attributes = vec![VertexAttribute::new("star_pos", VertexFormat::Float3)];

        let starfield_pipeline = ctx.new_pipeline(
            &[starfield_buffer_layout],
            &starfield_attributes,
            starfield_shader,
            starfield_pipeline_params,
        );

        let starfield_bindings = Bindings {
            vertex_buffers: vec![starfield_vertex_buffer],
            index_buffer: starfield_index_buffer,
            images: vec![],
        };

        Self {
            solid_pipeline,
            wireframe_pipeline,
            starfield_pipeline,
            starfield_bindings,
        }
    }

    pub fn draw(
        &mut self,
        ctx: &mut Box<dyn RenderingBackend>,
        mesh_manager: &mut MeshManager,
        view_proj: Mat4,
    ) {
        self.draw_starfield(ctx, view_proj);
        self.draw_solid_geometry(ctx, mesh_manager, view_proj);
        self.draw_line_geometry(ctx, mesh_manager, view_proj);

        mesh_manager.clear_instance_buffer();
    }

    fn draw_starfield(&self, ctx: &mut Box<dyn RenderingBackend>, view_proj: Mat4) {
        ctx.apply_pipeline(&self.starfield_pipeline);
        ctx.apply_bindings(&self.starfield_bindings);
        ctx.apply_uniforms(UniformsSource::table(&(view_proj)));
        ctx.draw(0, 2000, 1);
    }

    fn draw_line_geometry(
        &mut self,
        ctx: &mut Box<dyn RenderingBackend>,
        mesh_manager: &mut MeshManager,
        view_proj: Mat4,
    ) {
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
    }

    fn draw_solid_geometry(
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
    }
}
