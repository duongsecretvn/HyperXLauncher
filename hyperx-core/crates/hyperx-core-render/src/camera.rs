use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // in radians
    pub pitch: f32, // in radians
    pub fov_y: f32, // in radians
    pub z_near: f32,
    pub z_far: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    pub fn new(position: Vec3, aspect_ratio: f32) -> Self {
        Self {
            position,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            fov_y: 70.0f32.to_radians(),
            z_near: 0.1,
            z_far: 1000.0,
            aspect_ratio,
        }
    }

    pub fn forward(&self) -> Vec3 {
        let cos_pitch = self.pitch.cos();
        Vec3::new(
            self.yaw.cos() * cos_pitch,
            self.pitch.sin(),
            self.yaw.sin() * cos_pitch,
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward(), Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect_ratio, self.z_near, self.z_far)
    }

    pub fn view_proj_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Extract 6 frustum planes (Ax + By + Cz + D = 0) for view frustum culling
    pub fn frustum_planes(&self) -> [Vec4; 6] {
        let m = self.view_proj_matrix();
        let r0 = m.row(0);
        let r1 = m.row(1);
        let r2 = m.row(2);
        let r3 = m.row(3);

        let mut planes = [
            r3 + r0, // Left
            r3 - r0, // Right
            r3 + r1, // Bottom
            r3 - r1, // Top
            r3 + r2, // Near
            r3 - r2, // Far
        ];

        for p in &mut planes {
            let len = Vec3::new(p.x, p.y, p.z).length();
            if len > 0.0 {
                *p /= len;
            }
        }

        planes
    }

    /// Test if an AABB (min, max) is inside or intersecting the camera frustum
    pub fn is_box_visible(&self, min: Vec3, max: Vec3) -> bool {
        let planes = self.frustum_planes();
        for plane in &planes {
            let p_vertex = Vec3::new(
                if plane.x > 0.0 { max.x } else { min.x },
                if plane.y > 0.0 { max.y } else { min.y },
                if plane.z > 0.0 { max.z } else { min.z },
            );
            if plane.x * p_vertex.x + plane.y * p_vertex.y + plane.z * p_vertex.z + plane.w < 0.0 {
                return false;
            }
        }
        true
    }
}
