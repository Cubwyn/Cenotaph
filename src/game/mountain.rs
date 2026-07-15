use crate::data::world::level::{AtmosphereData, MountainReactionData};

#[derive(Debug, Clone)]
pub struct ActiveMountainReaction {
    profile: MountainReactionData,
    elapsed: f32,
}

impl ActiveMountainReaction {
    pub fn new(profile: MountainReactionData) -> Self {
        Self {
            profile,
            elapsed: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.elapsed = (self.elapsed + dt.max(0.0)).min(self.profile.duration);
        self.elapsed >= self.profile.duration
    }

    pub fn id(&self) -> &str {
        &self.profile.id
    }

    pub fn atmosphere(&self, base: &AtmosphereData) -> AtmosphereData {
        let intensity = self.intensity();
        let mut result = base.clone();

        blend_optional_color(&mut result.clear_color, self.profile.clear_color, intensity);
        blend_optional_color(&mut result.fog_color, self.profile.fog_color, intensity);
        blend_optional_color(
            &mut result.key_light_color,
            self.profile.key_light_color,
            intensity,
        );
        blend_optional_color(
            &mut result.particle_color,
            self.profile.particle_color,
            intensity,
        );
        blend_optional_vec3(&mut result.wind, self.profile.wind, intensity);

        result.fog_density *= lerp(1.0, self.profile.fog_density_multiplier, intensity).max(0.0);
        result.key_light_intensity *=
            lerp(1.0, self.profile.key_light_intensity_multiplier, intensity).max(0.0);
        result.particle_speed *= lerp(1.0, self.profile.particle_speed_multiplier, intensity);
        result.ambience_volume *=
            lerp(1.0, self.profile.ambience_volume_multiplier, intensity).max(0.0);
        if intensity > 0.01 {
            if let Some(preset) = self.profile.ambience_preset {
                result.ambience_preset = preset;
            }
        }

        result
    }

    fn intensity(&self) -> f32 {
        let duration = self.profile.duration.max(f32::EPSILON);
        let progress = (self.elapsed / duration).clamp(0.0, 1.0);
        (std::f32::consts::PI * progress).sin().max(0.0).powf(0.55)
    }
}

fn blend_optional_color(base: &mut [f32; 3], target: Option<[f32; 3]>, amount: f32) {
    blend_optional_vec3(base, target, amount);
}

fn blend_optional_vec3(base: &mut [f32; 3], target: Option<[f32; 3]>, amount: f32) {
    let Some(target) = target else {
        return;
    };
    for (value, target) in base.iter_mut().zip(target) {
        *value = lerp(*value, target, amount);
    }
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::world::level::AmbiencePreset;

    fn reaction() -> MountainReactionData {
        MountainReactionData {
            id: "answer".to_string(),
            duration: 4.0,
            clear_color: Some([0.2, 0.0, 0.0]),
            fog_color: Some([0.5, 0.1, 0.1]),
            fog_density_multiplier: 3.0,
            key_light_color: Some([1.0, 0.1, 0.1]),
            key_light_intensity_multiplier: 0.25,
            particle_color: Some([0.9, 0.2, 0.1]),
            particle_speed_multiplier: -1.0,
            wind: Some([-1.0, 0.0, 0.0]),
            ambience_preset: Some(AmbiencePreset::EmberVault),
            ambience_volume_multiplier: 1.5,
        }
    }

    #[test]
    fn reaction_peaks_then_restores_the_authored_atmosphere() {
        let base = AtmosphereData::default();
        let mut active = ActiveMountainReaction::new(reaction());

        active.tick(2.0);
        let peak = active.atmosphere(&base);
        assert!(peak.fog_density > base.fog_density * 2.9);
        assert!(peak.key_light_intensity < base.key_light_intensity * 0.3);
        assert!(peak.particle_speed < 0.0);
        assert_eq!(peak.ambience_preset, AmbiencePreset::EmberVault);

        assert!(active.tick(2.0));
        assert_eq!(active.atmosphere(&base), base);
    }
}
