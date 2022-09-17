
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + t * b
}

pub fn lerp_vec2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a * (1.0 - t) + t * b
}

pub fn lerp_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a * (1.0 - t) + t * b
}