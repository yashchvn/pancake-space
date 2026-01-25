use std::collections::HashSet;
use std::time::Instant;

use glam::{Quat, Vec2, Vec3, Vec4};
use hecs::{Entity, World};
use miniquad::{EventHandler, KeyCode, KeyMods, PassAction, RenderingBackend, window};
use rand::Rng;

use crate::components::flight::{
    AccelerationControlCommand, FlightController, TargetVelocity, ThrusterLimits,
};
use crate::components::navigation::NavigationTarget;
use crate::components::physics::{Forces, Inertia, Mass, Velocity};
use crate::components::render::Renderable;
use crate::components::transform::Transform;
use crate::render::camera::Camera;
use crate::render::mesh_batch::Instance;
use crate::render::mesh_manager::MeshManager;
use crate::render::renderer::*;
use crate::systems::flight_controller_system::flight_controller_system;
use crate::systems::navigation_system::navigation_system;
use crate::systems::physics_system::physics_system;
use crate::systems::thruster_system::thruster_system;

pub struct Stage {
    ctx: Box<dyn RenderingBackend>,
    mesh_manager: MeshManager,
    renderer: Renderer,
    camera: Camera,

    world: hecs::World,

    keys: HashSet<KeyCode>,
    mouse_pos: Vec2,
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
        let mouse_pos = Vec2::ZERO;
        let last_frame_time = Instant::now();

        let teapot_mesh_id = mesh_manager.register_mesh(&mut ctx, "src/assets/meshes/teapot.obj");
        let cow_mesh_id = mesh_manager.register_mesh(&mut ctx, "src/assets/meshes/cow.obj");
        let ship_mesh_id = mesh_manager.register_mesh(&mut ctx, "src/assets/meshes/ship1.obj");

        let player_entity = world.spawn((
            Transform {
                position: Vec3::ZERO,
                orientation: Quat::IDENTITY,
                scale: Vec3::new(0.01, 0.01, 0.01) * 0.0,
            },
            Renderable::new(ship_mesh_id),
            Mass::new(5000.0),
            Inertia::box_shape(5000.0, Vec3::new(3.0, 2.0, 5.0)),
            Velocity::ZERO,
            Forces::ZERO,
            ThrusterLimits::new(
                Vec3::new(25000.0, 25000.0, 50000.0) * 2.0, // ~1g lateral, ~2g forward
                Vec3::new(50000.0, 50000.0, 50000.0),       // Rotational control
            ),
            TargetVelocity::new(Vec3::ZERO, Vec3::ZERO),
            FlightController::new(1.0, 0.1, 10.0, 0.5),
            AccelerationControlCommand::new(),
            NavigationTarget::new(Vec3::new(0.0, 0.0, 0.0), None, 2.0),
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
                        Mass::new(1.0),
                        Inertia::box_shape(10.0, Vec3::new(2.0, 1.5, 5.0)),
                        Velocity::ZERO,
                        // Forces::new(
                        //     Vec3::new(
                        //         rand.random_range(-175.0..175.0),
                        //         rand.random_range(-175.0..175.0),
                        //         rand.random_range(-175.0..175.0),
                        //     ),
                        //     Vec3::new(
                        //         rand.random_range(-100.0..100.0),
                        //         rand.random_range(-100.0..100.0),
                        //         rand.random_range(-100.0..100.0),
                        //     ),
                        // ),
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
            mouse_pos,
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

        physics_system(&mut self.world, delta_time);
        navigation_system(&mut self.world);
        flight_controller_system(&mut self.world, delta_time);
        thruster_system(&mut self.world);

        if let Ok(transform) = self.world.get::<&Transform>(self.player_entity) {
            let local_offset = Vec3::new(4.0, 4.0, -10.0);

            let rotated_offset = transform.orientation * local_offset;

            self.camera.position = transform.position + rotated_offset;
            self.camera.up = transform.orientation * Vec3::Y;
            self.camera.target = transform.position;
        }

        let mut linear_move = Vec3::ZERO;
        if self.keys.contains(&KeyCode::W) {
            linear_move.z += 0.1;
        }
        if self.keys.contains(&KeyCode::A) {
            linear_move.x += 0.1;
        }
        if self.keys.contains(&KeyCode::S) {
            linear_move.z -= 0.1;
        }
        if self.keys.contains(&KeyCode::D) {
            linear_move.x -= 0.1;
        }
        if self.keys.contains(&KeyCode::Space) {
            linear_move.y += 0.1;
        }
        if self.keys.contains(&KeyCode::LeftShift) {
            linear_move.y -= 0.1;
        }

        // self.camera.position += cam_linear_move * 2.0;

        if let Ok(mut nav_target) = self.world.get::<&mut NavigationTarget>(self.player_entity) {
            nav_target.target_position += linear_move * 5.0;
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

    fn key_down_event(&mut self, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        self.keys.insert(keycode);
    }

    fn key_up_event(&mut self, keycode: KeyCode, _keymods: KeyMods) {
        self.keys.remove(&keycode);
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.mouse_pos = Vec2::new(x, y);
    }
}
