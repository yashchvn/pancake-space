use std::collections::HashSet;
use std::f32;
use std::time::Instant;

use glam::{Quat, Vec2, Vec3, Vec4};
use hecs::{Entity, World};
use miniquad::{EventHandler, KeyCode, KeyMods, PassAction, RenderingBackend, window};
use rand::Rng;

use crate::flight::flight_components::{
    AccelerationControlCommand, FlightController, TargetVelocity, ThrusterLimits,
};
use crate::flight::navigation_components::NavigationTarget;
use crate::flight::{
    flight_controller_system::flight_controller_system, navigation_system::navigation_system,
    thruster_system::thruster_system,
};

use crate::physics::physics_components::{BoxCollider, Forces, MassProperties, Velocity};
use crate::physics::physics_system::physics_system;
use crate::physics::physics_world::PhysicsWorld;
use crate::physics::sync_physics::{sync_ecs_to_rapier, sync_new_entities, sync_rapier_to_ecs};
use crate::physics::transform::Transform;
use crate::render::camera::Camera;
use crate::render::mesh_batch::Instance;
use crate::render::mesh_manager::MeshManager;
use crate::render::render_components::Renderable;
use crate::render::renderer::Renderer;

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
    elapsed_time: f32,

    player_entity: Entity,
}

impl Stage {
    pub fn new() -> Self {
        let mut ctx = window::new_rendering_backend();
        let renderer = Renderer::new(&mut ctx);
        let mesh_manager = MeshManager::new();
        let camera = Camera::new(800.0 / 600.0);
        let world = World::new();
        let physics_world = PhysicsWorld::new();
        let keys = HashSet::new();
        let mouse_pos = Vec2::ZERO;
        let last_frame_time = Instant::now();
        let elapsed_time = 0.0;
        let player_entity = Entity::DANGLING;

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
            elapsed_time,
            player_entity,
        }
    }

    pub fn init(&mut self) -> bool {
        // unsafe {
        //     glEnable(GL_PROGRAM_POINT_SIZE);
        // }

        let albatross_mesh_id = self
            .mesh_manager
            .register_mesh(&mut self.ctx, "src/assets/meshes/albatross.obj");
        let planet_mesh_id = self
            .mesh_manager
            .register_mesh(&mut self.ctx, "src/assets/meshes/planet.obj");
        let teapot_mesh_id = self
            .mesh_manager
            .register_mesh(&mut self.ctx, "src/assets/meshes/teapot.obj");

        let player_entity = self.world.spawn((
            Transform {
                position: Vec3::ZERO,
                orientation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Renderable::new(albatross_mesh_id),
            MassProperties::new(5000.0),
            BoxCollider::new(9.5484, 1.28, 4.3138),
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
        self.player_entity = player_entity;

        let mut rand = rand::rng();
        for i in 1..100 {
            let random_pos = Vec3::new(
                rand.random_range(-1.0..1.0),
                rand.random_range(-1.0..1.0),
                rand.random_range(-1.0..1.0),
            )
            .normalize()
                * 30.0;

            let random_target = Vec3::new(
                rand.random_range(-1.0..1.0),
                rand.random_range(-1.0..1.0),
                rand.random_range(-1.0..1.0),
            )
            .normalize()
                * 200.0;

            self.world.spawn((
                Transform {
                    position: random_pos,
                    orientation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                Renderable::new(albatross_mesh_id),
                MassProperties::new(5000.0),
                BoxCollider::new(9.5484, 1.28, 4.3138),
                Velocity::ZERO,
                Forces::ZERO,
                ThrusterLimits::new(
                    Vec3::new(25000.0, 25000.0, 50000.0) * 2.0,
                    Vec3::new(50000.0, 50000.0, 50000.0),
                ),
                TargetVelocity::new(Vec3::ZERO, Vec3::ZERO),
                FlightController::new(1.0, 0.1, 2.0, 0.5),
                AccelerationControlCommand::new(),
                NavigationTarget::new(random_target, Quat::IDENTITY, 2.0),
            ));
        }

        // self.world.spawn((
        //     Transform {
        //         position: Vec3::new(500.0, 500.0, 500.0),
        //         orientation: Quat::IDENTITY,
        //         scale: Vec3::ONE * 500.0,
        //     },
        //     Renderable::new(planet_mesh_id),
        // ));

        // let grid_scale = 5;
        // for i in 0..grid_scale {
        //     for j in 0..grid_scale {
        //         for k in 0..grid_scale {
        //             self.world.spawn((
        //                 Transform {
        //                     position: Vec3::new((i * 15) as f32, (j * 15) as f32, (k * 15) as f32),
        //                     orientation: Quat::IDENTITY,
        //                     scale: Vec3::ONE,
        //                 },
        //                 Renderable::new(teapot_mesh_id),
        //                 Mass::new(1.0),
        //                 Inertia::box_shape(10.0, Vec3::new(2.0, 1.5, 5.0)),
        //                 Velocity::ZERO,
        //                 Forces::new(
        //                     Vec3::new(
        //                         rand.random_range(-175.0..175.0),
        //                         rand.random_range(-175.0..175.0),
        //                         rand.random_range(-175.0..175.0),
        //                     ),
        //                     Vec3::new(
        //                         rand.random_range(-100.0..100.0),
        //                         rand.random_range(-100.0..100.0),
        //                         rand.random_range(-100.0..100.0),
        //                     ),
        //                 ),
        //             ));
        //         }
        //     }
        // }

        return true;
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.elapsed_time += delta_time;
        // println!("{}", 1.0 / delta_time);
        if delta_time < 1.0 / 70.0 {
            // todo: use accumulator instead of skipped frames
            return;
        }

        self.last_frame_time = now;

        // Use to stress test flight controller/thruster limits
        // for (_entity, (forces)) in self.world.query_mut::<&mut Forces>() {
        //     let mut rand = rand::rng();
        //     forces.torque += Vec3::new(
        //         rand.random_range(-100.0..100.0),
        //         rand.random_range(-100.0..100.0),
        //         rand.random_range(-100.0..100.0),
        //     ) * 800.0;

        //     forces.linear += Vec3::new(
        //         rand.random_range(-100.0..100.0),
        //         rand.random_range(-100.0..100.0),
        //         rand.random_range(-100.0..100.0),
        //     ) * 800.0;

        //     // if rand.random_bool(0.2) {
        //     //     forces.torque += Vec3::ONE * 80000.0;

        //     //     forces.linear += Vec3::ONE * 80000.0;
        //     // }
        // }

        navigation_system(&mut self.world);
        flight_controller_system(&mut self.world, delta_time);
        thruster_system(&mut self.world);

        sync_new_entities(&mut self.world, &mut self.physics_world);
        sync_ecs_to_rapier(&self.world, &mut self.physics_world);
        physics_system(&mut self.physics_world, delta_time);

        sync_rapier_to_ecs(&mut self.world, &mut self.physics_world);

        if let Ok(transform) = self.world.get::<&Transform>(self.player_entity) {
            if self.camera.position.distance(transform.position) > 10.0 {
                let direction = (self.camera.position - transform.position).normalize();
                self.camera.position = transform.position + direction * 10.0;
            }

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

        for (entity, nav_target) in self.world.query_mut::<&mut NavigationTarget>() {
            if self.player_entity == entity {
                continue;
            }

            let angle = self.elapsed_time * 0.5; // 0.5 is the speed, adjust as needed
            let radius = (((entity.id()) * 100) + 50) as f32; // adjust radius as needed

            nav_target.target_position = Vec3::new(0.0, radius * angle.cos(), radius * angle.sin());
        }
    }

    fn draw(&mut self) {
        for (_entity, (transform, render_comp)) in
            self.world.query::<(&Transform, &Renderable)>().iter()
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
