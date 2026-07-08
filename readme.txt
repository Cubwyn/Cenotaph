CENOTAPH: THE GREAT OMISSION
============================

Short Project Summary
Last Updated: July 1, 2026

Cenotaph is a surreal vertical looter RPG about surviving repeated ascents
through a hostile mountain that changes, forgets, and rewards adaptation.

Current state:
- Rust 2021 prototype.
- winit, wgpu, Rapier3D, and rodio.
- JSON level data and TOML config data.
- TOML enemy definition data under data/enemies.
- Foundation gameplay includes movement, jump, sprint, dash, pause, HUD,
  prototype shooting, baseline enemy chase/attack, hurtbox damage, death,
  respawn, prototype resource pickup/Anchor banking, and content validation.

Run commands:
- cargo run -- ashwalk_01
- cargo run -- foundation_test
- cargo run -- validate

Core checks:
- powershell -ExecutionPolicy Bypass -File scripts/foundation_check.ps1
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo test

Current north star:
First Ascent Prototype.

Current enemy direction:
The implemented content currently has a Burdened enemy prop backed by authored
low-poly enemy silhouettes under assets/enemies. Runtime loading fills enemy
health/collider/model data from data/enemies. The baseline enemy loop reads
activation range, movement, attack range, wind-up, damage, and cooldown from
the same data. The gameplay-first enemy roster is documented in
ENEMY_GAMEPLAY_ROSTER.txt, and the 3D model generator brief is documented in
ENEMY_MODEL_GENERATOR_BRIEF.txt.

Best source of truth:
- README.md for the master overview.
- FOUNDATION.md for the current stable groundwork.
- layout_guide.txt for the source tree.
