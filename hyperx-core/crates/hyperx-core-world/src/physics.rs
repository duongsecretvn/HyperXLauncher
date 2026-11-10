use glam::{DVec3, IVec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: DVec3,
    pub max: DVec3,
}

impl Aabb {
    pub fn new(min: DVec3, max: DVec3) -> Self {
        Self { min, max }
    }

    pub fn from_block(x: i32, y: i32, z: i32) -> Self {
        Self {
            min: DVec3::new(x as f64, y as f64, z as f64),
            max: DVec3::new(x as f64 + 1.0, y as f64 + 1.0, z as f64 + 1.0),
        }
    }

    pub fn offset(&self, d: DVec3) -> Self {
        Self {
            min: self.min + d,
            max: self.max + d,
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    pub fn calculate_x_offset(&self, other: &Self, mut offset: f64) -> f64 {
        if other.max.y > self.min.y
            && other.min.y < self.max.y
            && other.max.z > self.min.z
            && other.min.z < self.max.z
        {
            if offset > 0.0 && other.max.x <= self.min.x {
                let d = self.min.x - other.max.x;
                if d < offset {
                    offset = d;
                }
            } else if offset < 0.0 && other.min.x >= self.max.x {
                let d = self.max.x - other.min.x;
                if d > offset {
                    offset = d;
                }
            }
        }
        offset
    }

    pub fn calculate_y_offset(&self, other: &Self, mut offset: f64) -> f64 {
        if other.max.x > self.min.x
            && other.min.x < self.max.x
            && other.max.z > self.min.z
            && other.min.z < self.max.z
        {
            if offset > 0.0 && other.max.y <= self.min.y {
                let d = self.min.y - other.max.y;
                if d < offset {
                    offset = d;
                }
            } else if offset < 0.0 && other.min.y >= self.max.y {
                let d = self.max.y - other.min.y;
                if d > offset {
                    offset = d;
                }
            }
        }
        offset
    }

    pub fn calculate_z_offset(&self, other: &Self, mut offset: f64) -> f64 {
        if other.max.x > self.min.x
            && other.min.x < self.max.x
            && other.max.y > self.min.y
            && other.min.y < self.max.y
        {
            if offset > 0.0 && other.max.z <= self.min.z {
                let d = self.min.z - other.max.z;
                if d < offset {
                    offset = d;
                }
            } else if offset < 0.0 && other.min.z >= self.max.z {
                let d = self.max.z - other.min.z;
                if d > offset {
                    offset = d;
                }
            }
        }
        offset
    }
}

/// Raycast result
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub block_pos: IVec3,
    pub normal: IVec3,
    pub distance: f64,
}

/// Fast 3D Digital Differential Analyzer (DDA) raycast through voxels
pub fn raycast_voxel<F>(origin: DVec3, dir: DVec3, max_dist: f64, is_solid: F) -> Option<RaycastHit>
where
    F: Fn(i32, i32, i32) -> bool,
{
    let dir = dir.normalize();
    let mut current_block = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    let step_x = if dir.x > 0.0 { 1 } else { -1 };
    let step_y = if dir.y > 0.0 { 1 } else { -1 };
    let step_z = if dir.z > 0.0 { 1 } else { -1 };

    let delta_x = if dir.x != 0.0 { (1.0 / dir.x).abs() } else { f64::MAX };
    let delta_y = if dir.y != 0.0 { (1.0 / dir.y).abs() } else { f64::MAX };
    let delta_z = if dir.z != 0.0 { (1.0 / dir.z).abs() } else { f64::MAX };

    let mut side_x = if dir.x > 0.0 {
        (current_block.x as f64 + 1.0 - origin.x) * delta_x
    } else {
        (origin.x - current_block.x as f64) * delta_x
    };
    let mut side_y = if dir.y > 0.0 {
        (current_block.y as f64 + 1.0 - origin.y) * delta_y
    } else {
        (origin.y - current_block.y as f64) * delta_y
    };
    let mut side_z = if dir.z > 0.0 {
        (current_block.z as f64 + 1.0 - origin.z) * delta_z
    } else {
        (origin.z - current_block.z as f64) * delta_z
    };

    let mut normal = IVec3::ZERO;
    let mut dist = 0.0;

    while dist <= max_dist {
        if is_solid(current_block.x, current_block.y, current_block.z) {
            return Some(RaycastHit {
                block_pos: current_block,
                normal,
                distance: dist,
            });
        }

        if side_x < side_y && side_x < side_z {
            dist = side_x;
            side_x += delta_x;
            current_block.x += step_x;
            normal = IVec3::new(-step_x, 0, 0);
        } else if side_y < side_z {
            dist = side_y;
            side_y += delta_y;
            current_block.y += step_y;
            normal = IVec3::new(0, -step_y, 0);
        } else {
            dist = side_z;
            side_z += delta_z;
            current_block.z += step_z;
            normal = IVec3::new(0, 0, -step_z);
        }
    }

    None
}
