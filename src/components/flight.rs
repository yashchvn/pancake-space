use glam::Vec3;

pub struct ThrusterLimits {
    pub max_force: Vec3,
    pub max_torque: Vec3,
}

impl ThrusterLimits {
    pub fn new(max_force: Vec3, max_torque: Vec3) -> Self {
        Self {
            max_force,
            max_torque,
        }
    }
}

pub struct ControlTarget {
    pub target_linear_velocity: Vec3,
    pub target_angular_velocity: Vec3,
}

impl ControlTarget {
    pub fn new(linear_target: Vec3, angular_target: Vec3) -> Self {
        Self {
            target_linear_velocity: linear_target,
            target_angular_velocity: angular_target,
        }
    }
}

pub struct ControlCommand {
    pub linear_acceleration: Vec3,
    pub angular_acceleration: Vec3,
}

impl ControlCommand {
    pub fn new() -> Self {
        Self {
            linear_acceleration: Vec3::ZERO,
            angular_acceleration: Vec3::ZERO,
        }
    }
}

pub struct FlightController {
    // Linear gains
    pub lin_vel_kp: f32,
    pub lin_vel_ki: f32,
    pub lin_vel_kd: f32,
    pub lin_vel_error_integral: Vec3,
    pub lin_vel_prev_error: Vec3,

    // Angular gains
    pub ang_vel_kp: f32,
    pub ang_vel_ki: f32,
    pub ang_vel_kd: f32,
    pub ang_vel_error_integral: Vec3,
    pub ang_vel_prev_error: Vec3,
}

impl FlightController {
    pub fn new(
        lin_vel_kp: f32,
        lin_vel_ki: f32,
        lin_vel_kd: f32,
        ang_vel_kp: f32,
        ang_vel_ki: f32,
        ang_vel_kd: f32,
    ) -> Self {
        Self {
            lin_vel_kp,
            lin_vel_ki,
            lin_vel_kd,
            ang_vel_kp,
            ang_vel_ki,
            ang_vel_kd,

            lin_vel_error_integral: Vec3::ZERO,
            ang_vel_error_integral: Vec3::ZERO,
            lin_vel_prev_error: Vec3::ZERO,
            ang_vel_prev_error: Vec3::ZERO,
        }
    }
}
