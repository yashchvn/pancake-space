use hecs::World;

use crate::components::{
    flight::{ControlCommand, ControlTarget, FlightController},
    physics::Velocity,
};

pub fn flight_controller_system(world: &mut World, dt: f32) {
    for (_entity, (velocity, target, controller, command)) in world
        .query::<(
            &Velocity,
            &ControlTarget,
            &mut FlightController,
            &mut ControlCommand,
        )>()
        .iter()
    {
        // ----- Linear -----
        let lin_vel_error = target.target_linear_velocity - velocity.linear;
        // controller.lin_vel_error_integral *= 0.5;
        controller.lin_vel_error_integral += lin_vel_error * dt;
        let lin_vel_error_derivative = (lin_vel_error - controller.lin_vel_prev_error) / dt;
        controller.lin_vel_prev_error = lin_vel_error;

        let linear_acceleration = lin_vel_error * controller.lin_vel_kp
            + controller.lin_vel_error_integral * controller.lin_vel_ki
            + lin_vel_error_derivative * controller.lin_vel_kd;

        command.linear_acceleration = linear_acceleration;

        // ----- Angular -----
        let ang_vel_error = target.target_angular_velocity - velocity.angular;

        // controller.ang_vel_error_integral *= 0.5;
        controller.ang_vel_error_integral += ang_vel_error * dt;
        let ang_vel_error_derivative = (ang_vel_error - controller.ang_vel_prev_error) / dt;

        let angular_acceleration = ang_vel_error * controller.ang_vel_kp
            + controller.ang_vel_error_integral * controller.ang_vel_ki
            + ang_vel_error_derivative * controller.ang_vel_kd;

        command.angular_acceleration = angular_acceleration;

        // println!(
        //     "<{} {}>    <{} {}>",
        //     target.target_linear_velocity,
        //     target.target_angular_velocity,
        //     linear_acceleration,
        //     angular_acceleration
        // );
    }
}
