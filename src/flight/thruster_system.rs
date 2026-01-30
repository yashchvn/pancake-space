use hecs::World;

use crate::{
    flight::flight_components::{AccelerationControlCommand, ThrusterLimits},
    physics::{
        physics_components::{Forces, Inertia, Mass},
        transform::Transform,
    },
};

pub fn thruster_system(world: &mut World) {
    for (_entity, (transform, mass, inertia, forces, command, limits)) in world
        .query::<(
            &Transform,
            &Mass,
            &Inertia,
            &mut Forces,
            &AccelerationControlCommand,
            &ThrusterLimits,
        )>()
        .iter()
    {
        // ----- Linear -----
        let desired_force = command.linear_acceleration * mass.mass;

        // Transform to local space to apply limits
        let local_desired_force = transform.orientation.inverse() * desired_force;
        let clamped_local_force = local_desired_force.clamp(-limits.max_force, limits.max_force);

        // Transform back to world space
        let world_force = transform.orientation * clamped_local_force;
        forces.linear += world_force;

        // ----- Angular -----
        // Transform commanded angular acceleration to local space FIRST
        let local_angular_accel = transform.orientation.inverse() * command.angular_acceleration;

        // Now compute torque using local-space inertia tensor
        let local_desired_torque = inertia.inverse_tensor.inverse() * local_angular_accel;

        // Apply limits in local space
        let clamped_local_torque =
            local_desired_torque.clamp(-limits.max_torque, limits.max_torque);

        // Transform to world space for physics system
        let world_torque = transform.orientation * clamped_local_torque;
        forces.torque += world_torque;

        // println!(
        //     "Forces - Desired: {:.2} Clamped: {:.2} | Torque - Desired: {:.2} Clamped: {:.2}",
        //     local_desired_force, clamped_local_force, local_desired_torque, clamped_local_torque
        // );
    }
}
