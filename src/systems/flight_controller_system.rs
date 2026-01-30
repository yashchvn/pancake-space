use hecs::World;

use crate::components::{
    flight::{AccelerationControlCommand, FlightController, TargetVelocity},
    physics::Velocity,
};

pub fn flight_controller_system(world: &mut World, dt: f32) {
    for (_entity, (velocity, target, controller, command)) in world
        .query::<(
            &Velocity,
            &TargetVelocity,
            &mut FlightController,
            &mut AccelerationControlCommand,
        )>()
        .iter()
    {
        // ----- Linear -----
        let lin_vel_error = target.target_linear_velocity - velocity.linear;

        let lin_vel_derivative = -(velocity.linear - controller.lin_vel_prev) / dt;
        controller.lin_vel_prev = lin_vel_error;

        let linear_acceleration =
            lin_vel_error * controller.lin_vel_kp + lin_vel_derivative * controller.lin_vel_kd;

        command.linear_acceleration = linear_acceleration;

        // ----- Angular -----
        let ang_vel_error = target.target_angular_velocity - velocity.angular;

        let ang_vel_derivative = (ang_vel_error - controller.ang_vel_prev) / dt;
        controller.ang_vel_prev = ang_vel_error;

        let angular_acceleration =
            ang_vel_error * controller.ang_vel_kp + ang_vel_derivative * controller.ang_vel_kd;

        command.angular_acceleration = angular_acceleration;

        // println!(
        //     "P: {:.2}     |     D: {:.2}",
        //     ang_vel_error * controller.ang_vel_kp,
        //     ang_vel_derivative * controller.ang_vel_kd
        // );

        // println!(
        //     "<{:>07.2} {:>07.2}>         |         <{:>07.2} {:>07.2}>",
        //     target.target_linear_velocity,
        //     linear_acceleration,
        //     target.target_angular_velocity,
        //     angular_acceleration
        // );
    }
}
