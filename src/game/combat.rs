use glam::Vec3;

/// Returns true if the ray (`origin + t * dir`) passes within `radius` of `center`.
pub(crate) fn ray_hits_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> bool {
    let oc = center - origin;
    let proj = oc.dot(dir);
    if proj < 0.0 {
        return false;
    }
    let closest = origin + dir * proj;
    closest.distance_squared(center) < radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hits_sphere_in_front_of_origin() {
        assert!(ray_hits_sphere(
            Vec3::ZERO,
            Vec3::X,
            Vec3::new(3.0, 0.5, 0.0),
            1.0
        ));
    }

    #[test]
    fn ray_ignores_sphere_behind_origin() {
        assert!(!ray_hits_sphere(
            Vec3::ZERO,
            Vec3::X,
            Vec3::new(-1.0, 0.0, 0.0),
            1.0
        ));
    }
}
