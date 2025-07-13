use glam::{Mat4, Quat, Vec3};

pub struct Camera {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            aspect_ratio: 16.0 / 9.0,
            fov_y: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        }
    }
}

impl Camera {
    /// Gibt die View-Matrix zurück (Inverse der Kamera-Transformation)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation).inverse()
    }

    /// Gibt die Projektionsmatrix zurück
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    /// Kombiniert View und Projection (für Shader)
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.translation += translation;
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.scale *= scale;
    }
}
