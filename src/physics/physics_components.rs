use glam::{Mat3, Vec3};

pub struct Mass {
    pub mass: f32,
    pub inverse_mass: f32,
}

impl Mass {
    pub fn new(mass: f32) -> Self {
        Self {
            mass: mass,
            inverse_mass: 1.0 / mass,
        }
    }
}

pub struct Inertia {
    pub tensor: Mat3,
    pub inverse_tensor: Mat3,
}

impl Inertia {
    pub fn sphere(mass: f32, radius: f32) -> Self {
        let i = 0.4 * mass * radius * radius;

        let inverse_tensor = Mat3::from_diagonal(Vec3::splat(1.0 / i));

        Self {
            tensor: inverse_tensor.inverse(),
            inverse_tensor: inverse_tensor,
        }
    }

    pub fn box_shape(mass: f32, extents: Vec3) -> Self {
        let ix = (mass / 12.0) * (extents.y * extents.y + extents.z * extents.z);
        let iy = (mass / 12.0) * (extents.x * extents.x + extents.z * extents.z);
        let iz = (mass / 12.0) * (extents.x * extents.x + extents.y * extents.y);

        let inverse_tensor = Mat3::from_diagonal(Vec3::new(1.0 / ix, 1.0 / iy, 1.0 / iz));

        Self {
            tensor: inverse_tensor.inverse(),
            inverse_tensor: inverse_tensor,
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
