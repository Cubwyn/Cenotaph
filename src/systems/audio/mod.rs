// src/systems/audio/mod.rs
// Intentionally restrained placeholder audio.
//
// One-shot cue hooks remain available for future authored recordings, but they
// are silent. The runtime only produces non-tonal wind/pressure ambience.

use std::time::Duration;

use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::data::world::level::AmbiencePreset;

/// Semantic cue points reserved for future authored audio assets.
#[derive(Debug, Clone, Copy)]
pub enum SoundEffect {
    Footstep,
    Fire,
    Hit,
    Kill,
    Blocked,
    PlayerDamage,
    Dash,
    Jump,
    Land,
    LevelTransition,
    DeathSting,
    Pickup,
    Heal,
    MountainAnswer,
}

/// Owns the output stream and the current level ambience.
pub struct AudioSystem {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    ambient_sink: Option<Sink>,
    step_timer: f32,
    was_grounded: bool,
}

impl AudioSystem {
    pub fn new() -> Option<Self> {
        let (stream, stream_handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            stream_handle,
            ambient_sink: None,
            step_timer: 0.0,
            was_grounded: false,
        })
    }

    /// Replaces the current non-tonal atmosphere with a level-authored preset.
    pub fn set_ambience(&mut self, preset: AmbiencePreset, volume: f32) {
        if let Some(sink) = self.ambient_sink.take() {
            sink.stop();
        }
        if preset == AmbiencePreset::Silent || volume <= 0.0 {
            return;
        }
        let Ok(sink) = Sink::try_new(&self.stream_handle) else {
            return;
        };
        sink.append(AtmosphericNoise::with_preset(44_100, preset));
        sink.set_volume(volume.clamp(0.0, 1.0));
        self.ambient_sink = Some(sink);
    }

    pub fn pause_ambient(&self) {
        if let Some(sink) = self.ambient_sink.as_ref() {
            sink.pause();
        }
    }

    pub fn resume_ambient(&self) {
        if let Some(sink) = self.ambient_sink.as_ref() {
            sink.play();
        }
    }

    /// One-shot synthesis is deliberately disabled until authored recordings exist.
    pub fn play(&mut self, _effect: SoundEffect) {}

    /// Maintains movement cadence so recorded footsteps can be added without
    /// rebuilding movement/audio integration later.
    pub fn tick_movement(
        &mut self,
        dt: f32,
        movement_ratio: f32,
        is_sprinting: bool,
        grounded: bool,
        jumped: bool,
        landing_speed: f32,
    ) {
        let dt = dt.clamp(0.0, 0.1);
        self.step_timer = (self.step_timer - dt).max(0.0);

        if jumped {
            self.play(SoundEffect::Jump);
            self.step_timer = 0.22;
        }
        if grounded && !self.was_grounded && landing_speed > 2.0 {
            self.play(SoundEffect::Land);
            self.step_timer = 0.18;
        }
        self.was_grounded = grounded;

        if !grounded || movement_ratio < 0.08 {
            self.step_timer = self.step_timer.min(0.08);
            return;
        }
        if self.step_timer <= 0.0 {
            self.play(SoundEffect::Footstep);
            self.step_timer = movement_step_interval(movement_ratio, is_sprinting);
        }
    }
}

fn movement_step_interval(movement_ratio: f32, is_sprinting: bool) -> f32 {
    let base = if is_sprinting { 0.30 } else { 0.43 };
    base / movement_ratio.clamp(0.72, 1.45)
}

/// Continuous filtered noise with no oscillators, notes, bells, or periodic cues.
pub struct AtmosphericNoise {
    sample_rate: u32,
    sample_idx: u64,
    channels: u16,
    preset: AmbiencePreset,
    wind_state: [f32; 2],
    gust_state: [f32; 2],
    pressure_state: [f32; 2],
}

impl AtmosphericNoise {
    pub fn with_preset(sample_rate: u32, preset: AmbiencePreset) -> Self {
        Self {
            sample_rate,
            sample_idx: 0,
            channels: 2,
            preset,
            wind_state: [0.0; 2],
            gust_state: [0.0; 2],
            pressure_state: [0.0; 2],
        }
    }
}

impl Iterator for AtmosphericNoise {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.sample_idx / self.channels as u64;
        let channel = (self.sample_idx % self.channels as u64) as usize;
        let seed = frame
            .wrapping_mul(self.channels as u64)
            .wrapping_add(channel as u64);

        self.wind_state[channel] = self.wind_state[channel] * 0.982 + hash_noise(seed) * 0.018;
        self.gust_state[channel] = self.gust_state[channel] * 0.9994
            + hash_noise(seed.wrapping_mul(17).wrapping_add(41)) * 0.0006;
        self.pressure_state[channel] = self.pressure_state[channel] * 0.9997
            + hash_noise(seed.wrapping_mul(53).wrapping_add(97)) * 0.0003;

        let (wind, gust, pressure) = match self.preset {
            AmbiencePreset::Silent => (0.0, 0.0, 0.0),
            AmbiencePreset::Omission => (0.10, 0.10, 0.24),
            AmbiencePreset::AshWind => (0.54, 0.30, 0.10),
            AmbiencePreset::EmberVault => (0.20, 0.18, 0.28),
        };
        let sample = self.wind_state[channel] * wind
            + self.gust_state[channel] * gust
            + self.pressure_state[channel] * pressure;

        self.sample_idx = self.sample_idx.wrapping_add(1);
        Some((sample * 0.36).clamp(-0.9, 0.9))
    }
}

fn hash_noise(value: u64) -> f32 {
    let mut value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    (value as u32) as f32 / u32::MAX as f32 * 2.0 - 1.0
}

impl Source for AtmosphericNoise {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambience_presets_generate_finite_bounded_stereo_noise() {
        for preset in [
            AmbiencePreset::Omission,
            AmbiencePreset::AshWind,
            AmbiencePreset::EmberVault,
        ] {
            let samples = AtmosphericNoise::with_preset(44_100, preset)
                .take(20_000)
                .collect::<Vec<_>>();
            assert!(samples.iter().all(|sample| sample.is_finite()));
            assert!(samples.iter().all(|sample| sample.abs() <= 1.0));
            assert!(samples.iter().any(|sample| sample.abs() > 0.0001));
        }
    }

    #[test]
    fn silent_ambience_source_stays_silent() {
        assert!(
            AtmosphericNoise::with_preset(44_100, AmbiencePreset::Silent)
                .take(1024)
                .all(|sample| sample == 0.0)
        );
    }

    #[test]
    fn movement_cadence_accelerates_for_sprinting() {
        let walking = movement_step_interval(1.0, false);
        let sprinting = movement_step_interval(1.0, true);
        let faster = movement_step_interval(1.4, true);

        assert!(sprinting < walking);
        assert!(faster < sprinting);
    }
}
