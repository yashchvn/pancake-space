use glam::{Mat3, Vec3};

pub struct MassProperties {
    pub mass: f32,
    pub inverse_mass: f32,
}

impl MassProperties {
    pub fn new(mass: f32) -> Self {
        Self {
            mass: mass,
            inverse_mass: 1.0 / mass,
        }
    }
}

pub struct InertiaProperties {
    pub inertia: Mat3,
    pub inverse_inertia: Mat3,
}

impl InertiaProperties {
    pub fn new(inertia: Mat3) -> Self {
        Self {
            inertia: inertia,
            inverse_inertia: inertia.inverse(),
        }
    }
}

pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Velocity {
    pub const ZERO: Self = Self {
        linear: Vec3::ZERO,
        angular: Vec3::ZERO,
    };
}

pub struct Forces {
    pub linear: Vec3,
    pub torque: Vec3,
}

impl Forces {
    pub fn new(linear: Vec3, torque: Vec3) -> Self {
        Self { linear, torque }
    }

    pub const ZERO: Self = Self {
        linear: Vec3::ZERO,
        torque: Vec3::ZERO,
    };
}

pub struct BoxCollider {
    pub extents: Vec3,
}

impl BoxCollider {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            extents: Vec3::new(x, y, z),
        }
    }
}
