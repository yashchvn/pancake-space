use crate::core::physics_structures::PhysicsWorld;
use rapier3d::prelude::*;

pub fn physics_system(physics_world: &mut PhysicsWorld, dt: f32) {
    let gravity = vector![0.0, 0.0, 0.0];
    // let physics_hooks = ();
    // let event_handler = ();

    physics_world.integration_parameters.dt = dt;

    physics_world.physics_pipeline.step(
        &gravity,
        &physics_world.integration_parameters,
        &mut physics_world.islands,
        &mut physics_world.broad_phase,
        &mut physics_world.narrow_phase,
        &mut physics_world.bodies,
        &mut physics_world.colliders,
        &mut physics_world.impulse_joints,
        &mut physics_world.multibody_joints,
        &mut physics_world.ccd_solver,
        None,
        &(),
        &(),
    );
}
