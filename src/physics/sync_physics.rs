use glam::{Mat3, Quat, Vec3};
use hecs::World;
use rapier3d::{
    na::{Quaternion, UnitQuaternion},
    prelude::*,
};

use crate::physics::{
    physics_components::{BoxCollider, Forces, InertiaProperties, MassProperties, Velocity},
    physics_world::PhysicsWorld,
    transform::Transform,
};

pub fn sync_new_entities(world: &mut World, physics_world: &mut PhysicsWorld) {
    let mut new_entities = Vec::new();

    // Find entities with physics components but no RigidBodyHandle
    for (entity, (transform, mass_properties, box_collider, velocity)) in world
        .query::<(&Transform, &MassProperties, &BoxCollider, &Velocity)>()
        .without::<&RigidBodyHandle>() // Key filter!
        .iter()
    {
        // Create rigid body and collider (same logic as your current init_physics)
        let position = transform.position;
        let quat = transform.orientation;
        let na_quat =
            UnitQuaternion::from_quaternion(Quaternion::new(quat.w, quat.x, quat.y, quat.z));

        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![position.x, position.y, position.z])
            .rotation(na_quat.scaled_axis())
            .linvel(vector![
                velocity.linear.x,
                velocity.linear.y,
                velocity.linear.z
            ])
            .angvel(vector![
                velocity.angular.x,
                velocity.angular.y,
                velocity.angular.z
            ])
            .build();

        let collider = ColliderBuilder::cuboid(
            box_collider.extents.x / 2.0,
            box_collider.extents.y / 2.0,
            box_collider.extents.z / 2.0,
        )
        .mass(mass_properties.mass)
        .build();

        let inertia_tensor = collider.mass_properties().principal_inertia();
        let inertia_mat = Mat3::from_diagonal(Vec3::new(
            inertia_tensor.x,
            inertia_tensor.y,
            inertia_tensor.z,
        ));
        let inertia_properties = InertiaProperties::new(inertia_mat);

        let rb_handle = physics_world.bodies.insert(rb);
        let collider_handle = physics_world.colliders.insert_with_parent(
            collider,
            rb_handle,
            &mut physics_world.bodies,
        );

        new_entities.push((entity, rb_handle, collider_handle, inertia_properties));
    }

    // Insert handles into entities
    for (entity, rb_handle, collider_handle, inertia_properties) in new_entities {
        world
            .insert(entity, (rb_handle, collider_handle, inertia_properties))
            .expect("Entity should exist");
    }
}

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
