use glam::Vec3;
use hecs::World;

use crate::components::{
    flight::{TargetVelocity, ThrusterLimits},
    navigation::NavigationTarget,
    physics::Mass,
    transform::Transform,
};

pub fn navigation_system(world: &mut World) {
    for (_entity, (transform, nav_target, control_target, thruster_limits, mass)) in world
        .query::<(
            &Transform,
            &NavigationTarget,
            &mut TargetVelocity,
            &ThrusterLimits,
            &Mass,
        )>()
        .iter()
    {
        let to_target = nav_target.target_position - transform.position;
        let distance = to_target.length();

        if distance < nav_target.arrival_threshold {
            control_target.target_linear_velocity = Vec3::ZERO;
            continue;
        }

        let direction = to_target.normalize();

        let max_acceleration = calculate_max_acceleration_in_direction(
            direction,
            transform,
            thruster_limits,
            mass.mass,
        );

        control_target.target_linear_velocity =
            direction * (2.0 * max_acceleration * distance).sqrt();
    }
}

fn calculate_max_acceleration_in_direction(
    direction: Vec3,
    transform: &Transform,
    thruster_limits: &ThrusterLimits,
    mass: f32,
) -> f32 {
    // Transform the world-space direction into the ship's local frame
    let local_direction = transform.orientation.inverse() * direction;

    // Scale the direction by the thruster limits to find the point on the ellipsoid
    // The ellipsoid is defined by (x/a)² + (y/b)² + (z/c)² = 1
    // The maximum force in direction d is at the point where the ellipsoid intersects
    // the ray from origin in direction d
    let scaled = local_direction / thruster_limits.max_force;
    let scale_factor = 1.0 / scaled.length();

    let max_force_local = local_direction * scale_factor;
    let max_force_magnitude = max_force_local.length();

    max_force_magnitude / mass
}
