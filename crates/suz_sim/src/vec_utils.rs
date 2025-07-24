use bevy::math::Vec3;

#[inline]
pub fn f64_3_to_f32_3(input: &[f64; 3]) -> [f32; 3] {
    input.map(|p| p as f32)
}
#[inline]
pub fn f64_3_to_vec3(input: &[f64; 3]) -> Vec3 {
    input.map(|p| p as f32).into()
}
#[inline]
pub fn f32_3_to_f64_3(input: &[f32; 3]) -> [f64; 3] {
    input.map(|p| p as f64)
}
#[inline]
pub fn vec3_to_f64_3(input: Vec3) -> [f64; 3] {
    let arr: [f32; 3] = input.into();
    arr.map(|p| p as f64)
}

#[inline]
pub fn geodesic_distance(a: Vec3, b: Vec3) -> f32 {
    f32::acos(a.dot(b).clamp(-1., 1.))
}

#[inline]
pub fn geodesic_distance_arr(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let dot = if a.len() == 3 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    } else {
        // Forced by kdtree to have this be generic
        a.iter().zip(b.iter()).map(|(a, b)| *a * *b).sum::<f32>()
    };
    dot.clamp(-1.0, 1.0).acos()
}
