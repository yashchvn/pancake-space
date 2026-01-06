use std::collections::HashSet;
use std::time::Instant;

use glam::{Quat, Vec3, Vec4};
use hecs::{Entity, World};
use miniquad::{EventHandler, KeyCode, KeyMods, PassAction, RenderingBackend, window};
use rand::Rng;

use crate::components::flight::{ControlCommand, ControlTarget, FlightController, ThrusterLimits};
use crate::components::physics::{Forces, Inertia, Mass, Velocity};
use crate::components::render::Renderable;
use crate::components::transform::Transform;
use crate::render::mesh_batch::Instance;
use crate::render::mesh_manager::MeshManager;
use crate::render::renderer::*;
use crate::systems::flight_controller_system::flight_controller_system;
use crate::systems::physics_system::physics_system;
use crate::systems::thruster_system::thruster_system;

pub struct Stage {
    ctx: Box<dyn RenderingBackend>,
    mesh_manager: MeshManager,
    renderer: Renderer,
    camera: Camera,

    world: hecs::World,

    keys: HashSet<KeyCode>,

    last_frame_time: Instant,

    player_entity: Entity,
}

impl Stage {
    pub fn new() -> Self {
        let mut ctx = window::new_rendering_backend();
        let mut mesh_manager = MeshManager::new();
        let renderer = Renderer::new(&mut ctx);
        let camera = Camera::new(800.0 / 600.0);
        let mut world = World::new();
        let keys = HashSet::new();
        let last_frame_time = Instant::now();

        let teapot_mesh_id = mesh_manager.register_mesh(&mut ctx, "src/assets/meshes/teapot.obj");
        let cow_mesh_id = mesh_manager.register_mesh(&mut ctx, "src/assets/meshes/cow.obj");
        let ship_mesh_id = mesh_manager.register_mesh(&mut ctx, "src/assets/meshes/ship1.obj");

        let player_entity = world.spawn((
            Transform {
                position: Vec3::ZERO,
                orientation: Quat::IDENTITY,
                scale: Vec3::new(0.01, 0.01, 0.01),
            },
            Renderable::new(ship_mesh_id),
            Mass::new(10.0),
            Inertia::box_shape(10.0, Vec3::new(2.0, 1.5, 5.0)),
            Velocity::ZERO,
            Forces::ZERO,
            ThrusterLimits::new(
                Vec3::new(2000.0, 2000.0, 4000.0) * 1.1,
                Vec3::new(100.0, 100.0, 100.0),
            ),
            ControlTarget::new(Vec3::ZERO, Vec3::ZERO),
            FlightController::new(0.8, 0.0, 0.5, 1.2, 0.0, 0.4),
            ControlCommand::new(),
        ));

        let grid_scale = 10;
        let mut rand = rand::rng();
        for i in 0..grid_scale {
            for j in 0..grid_scale {
                for k in 0..grid_scale {
                    world.spawn((
                        Transform {
                            position: Vec3::new((i * 15) as f32, (j * 15) as f32, (k * 15) as f32),
                            orientation: Quat::IDENTITY,
                            scale: Vec3::ONE,
                        },
                        Renderable::new(teapot_mesh_id),
                        Mass::new(10.0),
                        Inertia::box_shape(10.0, Vec3::new(2.0, 1.5, 5.0)),
                        Velocity::ZERO,
                        Forces::new(
                            Vec3::new(
                                rand.random_range(0..75) as f32,
                                rand.random_range(0..75) as f32,
                                rand.random_range(0..75) as f32,
                            ),
                            Vec3::new(
                                rand.random_range(0..100) as f32,
                                rand.random_range(0..100) as f32,
                                rand.random_range(0..100) as f32,
                            ),
                        ),
                    ));
                }
            }
        }

        Self {
            ctx,
            mesh_manager,
            renderer,
            camera,
            world,
            keys,
            last_frame_time,
            player_entity,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        // println!("{}", 1.0 / delta_time);
        if delta_time < 1.0 / 70.0 {
            return;
        }

        self.last_frame_time = now;

        if let Ok((transform, target)) = self
            .world
            .query_one_mut::<(&Transform, &mut ControlTarget)>(self.player_entity)
        {
            let mut local_linear = Vec3::ZERO;
            let mut local_angular = Vec3::ZERO;

            if self.keys.contains(&KeyCode::W) {
                local_linear.z += 1.0;
            }
            if self.keys.contains(&KeyCode::S) {
                local_linear.z -= 1.0;
            }
            if self.keys.contains(&KeyCode::A) {
                local_linear.x += 1.0;
            }
            if self.keys.contains(&KeyCode::D) {
                local_linear.x -= 1.0;
            }

            if self.keys.contains(&KeyCode::I) {
                local_angular.x -= 1.0;
            }
            if self.keys.contains(&KeyCode::K) {
                local_angular.x += 1.0;
            }
            if self.keys.contains(&KeyCode::J) {
                local_angular.y += 1.0;
            }
            if self.keys.contains(&KeyCode::L) {
                local_angular.y -= 1.0;
            }
            if self.keys.contains(&KeyCode::E) {
                local_angular.z += 1.0;
            }
            if self.keys.contains(&KeyCode::Q) {
                local_angular.z -= 1.0;
            }

            let max_lin_speed = 100.0;
            let max_rot_speed = 10.0;

            // Convert local-space targets to world space
            target.target_linear_velocity = transform.orientation * (local_linear * max_lin_speed);
            target.target_angular_velocity =
                transform.orientation * (local_angular * max_rot_speed);
        }

        physics_system(&mut self.world, delta_time);
        flight_controller_system(&mut self.world, delta_time);
        thruster_system(&mut self.world);

        if let Ok(transform) = self.world.get::<&Transform>(self.player_entity) {
            let local_offset = Vec3::new(0.0, 2.0, -10.0);

            let rotated_offset = transform.orientation * local_offset;

            self.camera.position = transform.position + rotated_offset;
            self.camera.up = transform.orientation * Vec3::Y;
            self.camera.target = transform.position;
        }
    }

    fn draw(&mut self) {
        for (_entity, (transform, render_comp)) in
            self.world.query::<(&mut Transform, &Renderable)>().iter()
        {
            self.mesh_manager.submit_mesh_instance(
                Instance::new(transform.to_mat4(), Vec4::new(0.1, 0.1, 0.1, 1.0)),
                render_comp.mesh_id,
            );
        }

        self.ctx.begin_default_pass(PassAction::Clear {
            color: Some((0.0, 0.0, 0.0, 1.0)),
            depth: Some(1.0),
            stencil: None,
        });

        let view_proj = self.camera.view_projection();

        self.renderer
            .draw(&mut self.ctx, &mut self.mesh_manager, view_proj);

        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }

    fn key_down_event(&mut self, keycode: KeyCode, keymods: KeyMods, repeat: bool) {
        self.keys.insert(keycode);
    }

    fn key_up_event(&mut self, keycode: KeyCode, _keymods: KeyMods) {
        self.keys.remove(&keycode);
    }
}
