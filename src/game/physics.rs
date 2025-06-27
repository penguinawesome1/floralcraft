use glam::{ Vec2, Vec3 };

pub type Position2D = Vec2;
pub type Velocity2D = Vec2;
pub type Direction2D = Vec2;

pub type Position3D = Vec3;
pub type Velocity3D = Vec3;
pub type Direction3D = Vec3;

pub struct Hitbox2D {
    pub pos: Position2D,
    pub width: f32,
    pub height: f32,
}

impl Hitbox2D {
    pub const ZERO: Self = Self {
        pos: Position2D::ZERO,
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(pos: Position2D, width: f32, height: f32) -> Self {
        debug_assert!(width >= 0.0, "Hitbox width cannot be negative");
        debug_assert!(height >= 0.0, "Hitbox height cannot be negative");

        Self { pos, width, height }
    }

    /// Returns a bool for if the two hitboxes are overlapping.
    /// Uses AABB collision detection.
    pub const fn intersects(&self, other: &Self) -> bool {
        self.pos.x < other.pos.x + other.width &&
            self.pos.x + self.width > other.pos.x &&
            self.pos.y < other.pos.y + other.height &&
            self.pos.y + self.height > other.pos.y
    }
}

pub struct Hitbox3D {
    pub pos: Position3D,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

impl Hitbox3D {
    pub const ZERO: Self = Self {
        pos: Position3D::ZERO,
        width: 0.0,
        height: 0.0,
        depth: 0.0,
    };

    pub const fn new(pos: Position3D, width: f32, height: f32, depth: f32) -> Self {
        debug_assert!(width >= 0.0, "Hitbox width cannot be negative");
        debug_assert!(height >= 0.0, "Hitbox height cannot be negative");
        debug_assert!(depth >= 0.0, "Hitbox depth cannot be negative");

        Self { pos, width, height, depth }
    }

    /// Returns a bool for if the two hitboxes are overlapping.
    /// Uses AABB collision detection.
    pub const fn intersects(&self, other: &Self) -> bool {
        self.pos.x < other.pos.x + other.width &&
            self.pos.x + self.width > other.pos.x &&
            self.pos.y < other.pos.y + other.height &&
            self.pos.y + self.height > other.pos.y &&
            self.pos.z < other.pos.z + other.depth &&
            self.pos.z + self.depth > other.pos.z
    }
}
