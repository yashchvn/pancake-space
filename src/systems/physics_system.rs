use glam::{Mat3, Quat, Vec3};
use hecs::World;

use crate::components::{
    physics::{Forces, Inertia, Mass, Velocity},
    transform::Transform,
};

pub fn physics_system(world: &mut World, dt: f32) {
    for (_entity, (transform, mass, inertia, velocity, forces)) in
        world.query_mut::<(&mut Transform, &Mass, &Inertia, &mut Velocity, &mut Forces)>()
    {
        let linear_acceleration = forces.linear * mass.inverse_mass;
        velocity.linear += linear_acceleration * dt;

        // p = p + v * dt
        transform.position += velocity.linear * dt;

        // Angular motion integration
        // Convert inertia tensor from local space to world space
        let rotation_matrix = Mat3::from_quat(transform.orientation);
        let inverse_inertia_world =
            rotation_matrix * inertia.inverse_tensor * rotation_matrix.transpose();

        // ω = ω + (I^-1 * τ) * dt
        let angular_acceleration = inverse_inertia_world * forces.torque;
        velocity.angular += angular_acceleration * dt;

        // Update rotation using quaternion integration
        // q = q + 0.5 * (ω as quat) * q * dt
        let omega_quat = Quat::from_xyzw(
            velocity.angular.x,
            velocity.angular.y,
            velocity.angular.z,
            0.0,
        );
        let q_dot = omega_quat * transform.orientation * 0.5;
        transform.orientation = (transform.orientation + q_dot * dt).normalize();

        // Clear forces for next frame (or apply damping)
        forces.linear = Vec3::ZERO;
        forces.torque = Vec3::ZERO;

        // Optional: Apply damping to prevent perpetual motion
        // velocity.linear *= 0.97;
        // velocity.angular *= 0.97;
    }
}
