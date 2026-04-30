// src/world/strata.rs
// Defines the playable biomes (strata) of the Pillar of Regret.
// Each stratum has a unique atmosphere, palette, traversal identity, and
// sound signature — these values inform art, audio, and gameplay systems.
//
// NOTE: Strata are defined here as data; they will be wired into the
// level-loading and atmosphere systems as those are built out.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// A single vertical biome layer of the mountain.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StratumId {
    AshWalk,
    WardOfIrons,
    HangingSlums,
    Sanctuary,
    GalleryOfWind,
    MirrorCrust,
    TheBreach,
}

/// Runtime descriptor for a stratum — loaded alongside level data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stratum {
    pub id: StratumId,
    pub display_name: String,
    pub theme: String,
    pub color_palette: String,
    pub sound_identity: String,
    /// Vertical height range within the world (bottom, top).
    pub height_range: [f32; 2],
}

impl Stratum {
    /// Returns the canonical descriptor for every stratum in the game.
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                id: StratumId::AshWalk,
                display_name: "The Ash-Walk".to_string(),
                theme: "Collapse".to_string(),
                color_palette: "Sepia, pale red, smoke gray".to_string(),
                sound_identity: "Low wind, distant collapse".to_string(),
                height_range: [0.0, 200.0],
            },
            Self {
                id: StratumId::WardOfIrons,
                display_name: "The Ward of Irons".to_string(),
                theme: "Industrial insanity".to_string(),
                color_palette: "Rust red, oil black, tarnished brass".to_string(),
                sound_identity: "Steam bursts, grinding metal".to_string(),
                height_range: [200.0, 450.0],
            },
            Self {
                id: StratumId::HangingSlums,
                display_name: "The Hanging Slums".to_string(),
                theme: "Precarious life".to_string(),
                color_palette: "Desaturated teal, faded wood tones".to_string(),
                sound_identity: "Distant choral fragments, creaking chains".to_string(),
                height_range: [450.0, 700.0],
            },
            Self {
                id: StratumId::Sanctuary,
                display_name: "The Sanctuary".to_string(),
                theme: "Broken divinity".to_string(),
                color_palette: "Gold leaf, ivory stone, dim violet light".to_string(),
                sound_identity: "Cathedral reverb, bell resonance".to_string(),
                height_range: [700.0, 800.0],
            },
            Self {
                id: StratumId::GalleryOfWind,
                display_name: "The Gallery of Wind".to_string(),
                theme: "Breath of the abyss".to_string(),
                color_palette: "Pale limestone, wind-swept silver".to_string(),
                sound_identity: "Howling updrafts, carved trumpet resonance".to_string(),
                height_range: [800.0, 1050.0],
            },
            Self {
                id: StratumId::MirrorCrust,
                display_name: "The Mirror-Crust".to_string(),
                theme: "Reality distortion".to_string(),
                color_palette: "Silver-blue, glass white".to_string(),
                sound_identity: "Inverted echoes, crystalline tones".to_string(),
                height_range: [1050.0, 1300.0],
            },
            Self {
                id: StratumId::TheBreach,
                display_name: "The Breach".to_string(),
                theme: "Geometric collapse".to_string(),
                color_palette: "Black, stark white, color draining".to_string(),
                sound_identity: "Silence punctuated by fracture sounds".to_string(),
                height_range: [1300.0, 1500.0],
            },
        ]
    }
}
