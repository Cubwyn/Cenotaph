use glam::Vec3;

/// Returns the distance along the ray to the closest hit point on a sphere.
pub(crate) fn ray_sphere_hit_distance(
    origin: Vec3,
    dir: Vec3,
    center: Vec3,
    radius: f32,
    max_range: f32,
) -> Option<f32> {
    let dir_len_sq = dir.length_squared();
    if dir_len_sq <= f32::EPSILON {
        return None;
    }
    let dir = dir / dir_len_sq.sqrt();
    let radius = radius.max(0.0);
    let max_range = max_range.max(0.0);
    let oc = center - origin;
    let proj = oc.dot(dir);
    if proj < 0.0 {
        return None;
    }
    let closest = origin + dir * proj;
    let lateral_distance_sq = closest.distance_squared(center);
    if lateral_distance_sq > radius * radius {
        return None;
    }

    let half_chord = (radius * radius - lateral_distance_sq).sqrt();
    let entry_distance = (proj - half_chord).max(0.0);
    (entry_distance <= max_range).then_some(entry_distance)
}

/// Selects the closest target whose sphere intersects the ray.
pub(crate) fn closest_ray_sphere_hit<I>(
    origin: Vec3,
    dir: Vec3,
    targets: I,
    radius: f32,
    max_range: f32,
) -> Option<usize>
where
    I: IntoIterator<Item = (usize, Vec3)>,
{
    targets
        .into_iter()
        .filter_map(|(index, center)| {
            ray_sphere_hit_distance(origin, dir, center, radius, max_range)
                .map(|distance| (index, distance))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hits_sphere_in_front_of_origin() {
        assert!(
            ray_sphere_hit_distance(Vec3::ZERO, Vec3::X, Vec3::new(3.0, 0.5, 0.0), 1.0, 10.0)
                .is_some()
        );
    }

    #[test]
    fn ray_ignores_sphere_behind_origin() {
        assert_eq!(
            ray_sphere_hit_distance(Vec3::ZERO, Vec3::X, Vec3::new(-1.0, 0.0, 0.0), 1.0, 10.0),
            None
        );
    }

    #[test]
    fn zero_direction_ray_misses() {
        assert_eq!(
            ray_sphere_hit_distance(Vec3::ZERO, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 1.0, 10.0),
            None
        );
    }

    #[test]
    fn ray_hit_distance_accepts_unnormalized_direction() {
        assert!(ray_sphere_hit_distance(
            Vec3::ZERO,
            Vec3::X * 2.0,
            Vec3::new(3.0, 0.0, 0.0),
            1.0,
            10.0
        )
        .is_some());
    }

    #[test]
    fn ray_hit_respects_max_range() {
        assert_eq!(
            ray_sphere_hit_distance(Vec3::ZERO, Vec3::X, Vec3::new(10.0, 0.0, 0.0), 1.0, 5.0),
            None
        );
    }

    #[test]
    fn closest_ray_sphere_hit_prefers_nearest_target() {
        let targets = [
            (7, Vec3::new(10.0, 0.0, 0.0)),
            (3, Vec3::new(4.0, 0.0, 0.0)),
            (9, Vec3::new(6.0, 0.0, 0.0)),
        ];

        assert_eq!(
            closest_ray_sphere_hit(Vec3::ZERO, Vec3::X, targets, 1.0, 20.0),
            Some(3)
        );
    }
}
