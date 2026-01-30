use glam::Vec3;

// Maximum +/- linear and angular forces along XYZ axes
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

pub struct TargetVelocity {
    pub target_linear_velocity: Vec3,
    pub target_angular_velocity: Vec3,
}

impl TargetVelocity {
    pub fn new(linear_target: Vec3, angular_target: Vec3) -> Self {
        Self {
            target_linear_velocity: linear_target,
            target_angular_velocity: angular_target,
        }
    }
}

pub struct AccelerationControlCommand {
    pub linear_acceleration: Vec3,
    pub angular_acceleration: Vec3,
}

impl AccelerationControlCommand {
    pub fn new() -> Self {
        Self {
            linear_acceleration: Vec3::ZERO,
            angular_acceleration: Vec3::ZERO,
        }
    }
}

pub struct FlightController {
    // Consider separating tuning constants to separate shared struct and only keep per-ship state variables (error)
    // Linear gains
    pub lin_vel_kp: f32,
    pub lin_vel_kd: f32,
    pub lin_vel_prev: Vec3,

    // Angular gains
    pub ang_vel_kp: f32,
    pub ang_vel_kd: f32,
    pub ang_vel_prev: Vec3,
}

impl FlightController {
    pub fn new(lin_vel_kp: f32, lin_vel_kd: f32, ang_vel_kp: f32, ang_vel_kd: f32) -> Self {
        Self {
            lin_vel_kp,

            lin_vel_kd,
            ang_vel_kp,

            ang_vel_kd,

            lin_vel_prev: Vec3::ZERO,
            ang_vel_prev: Vec3::ZERO,
        }
    }
}
