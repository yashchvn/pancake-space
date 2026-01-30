use std::collections::HashSet;
use std::f32;
use std::time::Instant;

use glam::{Quat, Vec2, Vec3, Vec4};
use hecs::{Entity, World};
use miniquad::gl::{GL_PROGRAM_POINT_SIZE, glEnable};
use miniquad::{EventHandler, KeyCode, KeyMods, PassAction, RenderingBackend, window};

use crate::components::flight::{
    AccelerationControlCommand, FlightController, TargetVelocity, ThrusterLimits,
};
use crate::components::navigation::NavigationTarget;
use crate::components::physics::{Forces, Inertia, Mass, Velocity};
use crate::components::render::Renderable;
use crate::components::transform::Transform;
use crate::core::physics_structures::PhysicsWorld;
use crate::render::camera::Camera;
use crate::render::mesh_batch::Instance;
use crate::render::mesh_manager::MeshManager;
use crate::render::renderer::*;
use crate::systems::flight_controller_system::flight_controller_system;
use crate::systems::navigation_system::navigation_system;
use crate::systems::physics_system::physics_system;
use crate::systems::sync_physics::{sync_ecs_to_rapier, sync_rapier_to_ecs};
use crate::systems::thruster_system::thruster_system;

pub struct Stage {
    ctx: Box<dyn RenderingBackend>,
    mesh_manager: MeshManager,
    renderer: Renderer,
    camera: Camera,

    world: hecs::World,
    physics_world: PhysicsWorld,

    keys: HashSet<KeyCode>,
    mouse_pos: Vec2,
    last_frame_time: Instant,

    player_entity: Entity,
}

impl Stage {
    pub fn new() -> Self {
        unsafe {
            glEnable(GL_PROGRAM_POINT_SIZE);
        }

        let mut ctx = window::new_rendering_backend();
        let mut mesh_manager = MeshManager::new();
        let renderer = Renderer::new(&mut ctx);
        let camera = Camera::new(800.0 / 600.0);
        let mut world = World::new();
        let mut physics_world = PhysicsWorld::new();
        let keys = HashSet::new();
        let mouse_pos = Vec2::ZERO;
        let last_frame_time = Instant::now();

        let albatross_mesh_id =
            mesh_manager.register_mesh(&mut ctx, "src/assets/meshes/albatross.obj");
        let planet_mesh_id = mesh_manager.register_mesh(&mut ctx, "src/assets/meshes/planet.obj");

        // todo: create helpers for spawning entities with physics/other
        // create init system which builds rapier rigidbody + collider for all entities with certain ecs components
        let player_rb = RigidBodyBuilder::dynamic().build();
        let player_collider = ColliderBuilder::cuboid(9.5484 / 2.0, 1.28 / 2.0, 4.3138 / 2.0)
            .density(5000.0 / (9.5484 * 1.28 * 4.3138))
            .build();
        let player_rb_handle = physics_world.bodies.insert(player_rb);
        let player_collider_handle = physics_world.colliders.insert_with_parent(
            player_collider,
            player_rb_handle,
            &mut physics_world.bodies,
        );

        let player_entity = world.spawn((
            Transform {
                position: Vec3::ZERO,
                orientation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Renderable::new(albatross_mesh_id),
            player_rb_handle,
            Mass::new(5000.0),
            Inertia::box_shape(5000.0, Vec3::new(9.5484, 1.28, 4.3138)),
            Velocity::ZERO,
            Forces::ZERO,
            ThrusterLimits::new(
                Vec3::new(25000.0, 25000.0, 50000.0) * 2.0,
                Vec3::new(50000.0, 50000.0, 50000.0),
            ),
            TargetVelocity::new(Vec3::ZERO, Vec3::ZERO),
            FlightController::new(1.0, 0.1, 2.0, 0.5),
            AccelerationControlCommand::new(),
            NavigationTarget::new(Vec3::new(0.0, 0.0, 0.0), Quat::IDENTITY, 2.0),
        ));

        world.spawn((
            Transform {
                position: Vec3::new(500.0, 500.0, 500.0),
                orientation: Quat::IDENTITY,
                scale: Vec3::ONE * 500.0,
            },
            Renderable::new(planet_mesh_id),
        ));

        // let grid_scale = 10;
        // let mut rand = rand::rng();
        // for i in 0..grid_scale {
        //     for j in 0..grid_scale {
        //         for k in 0..grid_scale {
        //             world.spawn((
        //                 Transform {
        //                     position: Vec3::new((i * 15) as f32, (j * 15) as f32, (k * 15) as f32),
        //                     orientation: Quat::IDENTITY,
        //                     scale: Vec3::ONE,
        //                 },
        //                 Renderable::new(teapot_mesh_id),
        //                 Mass::new(1.0),
        //                 Inertia::box_shape(10.0, Vec3::new(2.0, 1.5, 5.0)),
        //                 Velocity::ZERO,
        //                 // Forces::new(
        //                 //     Vec3::new(
        //                 //         rand.random_range(-175.0..175.0),
        //                 //         rand.random_range(-175.0..175.0),
        //                 //         rand.random_range(-175.0..175.0),
        //                 //     ),
        //                 //     Vec3::new(
        //                 //         rand.random_range(-100.0..100.0),
        //                 //         rand.random_range(-100.0..100.0),
        //                 //         rand.random_range(-100.0..100.0),
        //                 //     ),
        //                 // ),
        //             ));
        //         }
        //     }
        // }

        Self {
            ctx,
            mesh_manager,
            renderer,
            camera,
            world,
            physics_world,
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

        // Use to stress test flight controller/thruster limits
        // if let Ok(mut forces) = self.world.get::<&mut Forces>(self.player_entity) {
        //     let mut rand = rand::rng();
        //     // forces.torque += Vec3::new(
        //     //     rand.random_range(-100.0..100.0),
        //     //     rand.random_range(-100.0..100.0),
        //     //     rand.random_range(-100.0..100.0),
        //     // ) * 800.0;

        //     // forces.linear += Vec3::new(
        //     //     rand.random_range(-100.0..100.0),
        //     //     rand.random_range(-100.0..100.0),
        //     //     rand.random_range(-100.0..100.0),
        //     // ) * 800.0;

        //     if rand.random_bool(0.2) {
        //         forces.torque += Vec3::ONE * 80000.0;

        //         forces.linear += Vec3::ONE * 80000.0;
        //     }
        // }

        navigation_system(&mut self.world);
        flight_controller_system(&mut self.world, delta_time);
        thruster_system(&mut self.world);

        sync_ecs_to_rapier(&self.world, &mut self.physics_world);
        physics_system(&mut self.physics_world, delta_time);

        sync_rapier_to_ecs(&mut self.world, &mut self.physics_world);

        if let Ok(transform) = self.world.get::<&Transform>(self.player_entity) {
            // let local_offset = Vec3::new(4.0, 4.0, -10.0);
            // let rotated_offset = transform.orientation * local_offset;
            // self.camera.position = transform.position + rotated_offset;
            // self.camera.up = transform.orientation * Vec3::Y;
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

        let mut delta_rot = Quat::IDENTITY;
        if self.keys.contains(&KeyCode::I) {
            delta_rot *= Quat::from_rotation_x(0.01);
        }
        if self.keys.contains(&KeyCode::K) {
            delta_rot *= Quat::from_rotation_x(-0.01);
        }
        if self.keys.contains(&KeyCode::J) {
            delta_rot *= Quat::from_rotation_y(0.01);
        }
        if self.keys.contains(&KeyCode::L) {
            delta_rot *= Quat::from_rotation_y(-0.01);
        }
        if self.keys.contains(&KeyCode::E) {
            delta_rot *= Quat::from_rotation_z(0.01);
        }
        if self.keys.contains(&KeyCode::Q) {
            delta_rot *= Quat::from_rotation_z(-0.01);
        }

        if let Ok(mut nav_target) = self.world.get::<&mut NavigationTarget>(self.player_entity) {
            nav_target.target_position += linear_move * 2.0;

            nav_target.target_orientation = (nav_target.target_orientation * delta_rot).normalize();
        }

        if self.keys.contains(&KeyCode::M) {
            if let Ok(mut nav_target) = self.world.get::<&mut NavigationTarget>(self.player_entity)
            {
                nav_target.target_orientation = Quat::IDENTITY;
            }
        }
    }

    fn draw(&mut self) {
        for (_entity, (transform, render_comp)) in
            self.world.query::<(&mut Transform, &Renderable)>().iter()
        {
            self.mesh_manager.submit_mesh_instance(
                Instance::new(transform.to_mat4(), Vec4::new(0.0, 0.0, 0.0, 1.0)),
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
