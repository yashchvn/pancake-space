use glam::{Quat, Vec3};
use hecs::World;
use rapier3d::prelude::*;

use crate::{
    components::{
        physics::{Forces, Velocity},
        transform::Transform,
    },
    core::physics_structures::PhysicsWorld,
};

pub fn sync_ecs_to_rapier(world: &World, physics_world: &mut PhysicsWorld) {
    for (_entity, (forces, rb_handle)) in world.query::<(&Forces, &RigidBodyHandle)>().iter() {
        if let Some(rb) = physics_world.bodies.get_mut(*rb_handle) {
            rb.add_force(
                vector![forces.linear.x, forces.linear.y, forces.linear.z],
                true,
            );
            rb.add_torque(
                vector![forces.torque.x, forces.torque.y, forces.torque.z],
                true,
            );
        }
    }
}

pub fn sync_rapier_to_ecs(world: &mut World, physics_world: &mut PhysicsWorld) {
    for (_entity, (transform, velocity, forces, rb_handle)) in
        world.query_mut::<(&mut Transform, &mut Velocity, &mut Forces, &RigidBodyHandle)>()
    {
        if let Some(rb) = physics_world.bodies.get_mut(*rb_handle) {
            let pos = rb.translation();
            let rot = rb.rotation();
            transform.position = Vec3::new(pos.x, pos.y, pos.z);
            transform.orientation = Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w);

            let lin_vel = rb.linvel();
            let ang_vel = rb.angvel();
            velocity.linear = Vec3::new(lin_vel.x, lin_vel.y, lin_vel.z);
            velocity.angular = Vec3::new(ang_vel.x, ang_vel.y, ang_vel.z);

            rb.reset_forces(true);
            rb.reset_torques(true);
            forces.linear = Vec3::ZERO;
            forces.torque = Vec3::ZERO;
        }
    }
}
