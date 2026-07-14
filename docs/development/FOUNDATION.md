# Cenotaph Foundation Stable Notes

## Purpose

This file documents the current stable groundwork target for Cenotaph.

The foundation build is not the full First Ascent Prototype. It is the usable
base underneath it: launch, load a level, move, pause, render, collide, take
damage, shoot a test enemy, respawn, validate content, and iterate safely.

## Current Runtime Shape

```text
main.rs
  -> app.rs
    -> EngineState
      -> data::config
      -> data::world
      -> systems::input
      -> systems::physics
      -> systems::render
      -> systems::audio
      -> game::player
      -> game::progression
```

`App` owns the winit event loop and OS-facing events.

`EngineState` owns GPU state, runtime subsystems, loaded level data, and frame
orchestration.

`game::player::PlayerState` owns player health, stamina, sprint state, hit
feedback, death, and respawn timers.

`game::progression::RunProgress` owns unsecured resource, banked resource, and
the active local Anchor respawn point.

## Stable Foundation Features

- Level loading from `levels/*.json`.
- Config loading from `config/tuning.toml` and `config/bindings.toml`.
- Enemy definition validation from `data/enemies/*.toml`.
- Low-poly authored enemy silhouette assets under `assets/enemies/`.
- First-person movement with jump, coyote time, sprint stamina, and dash.
- Pause mode through Escape, including cursor release and ambient pause/resume.
- HUD health/stamina bars, guide strip, thicker readable block text, crosshair,
  shot-result flashes, and pause overlay.
- Standalone localhost level editor launched with `cargo run -- editor`, with
  project level browsing, Hammer-style Camera/Top/Front/Side view panels, a 3D
  WebGL viewport, fly camera, ray-picked and orthographic placement/move/select
  tools, orthographic marquee and multi-prop selection, axis-constrained group
  transforms, snap controls, transactional undo/redo, drawable floor/wall/block/
  slope/cylinder/stair/terrain geometry brushes, reusable prop prefabs,
  browser-local draft recovery, tabbed authoring workspaces, palette, asset
  browser, configurable keybindings, right-click context commands,
  duplicate/copy/paste, focus-selected, guided system inspectors, Rust-backed
  validation, and direct save-to-level-file workflow.
- Project asset catalog for runtime-supported models plus source files across
  model, texture, material, audio, dialogue/data, level, and config formats.
- Editor server hardening: loopback-only bind, per-launch tokenized URL,
  token-required API calls, local Host/Origin checks, static embedded assets,
  path-restricted level/prefab writes, and backup-before-overwrite saves.
- In-game quick-adjust mode for runtime smoke checks, hot-reload verification,
  snapped placement, prop deletion, and emergency level edits.
- Static, sphere, mesh, hurtbox, enemy, decorative, trigger, path-bound, and
  dialogue/event-linked prop data.
- Level authoring graph data for asset imports, loot tables, paths, events, and
  dialogue blocks.
- Runtime level events for on-enter/proximity triggers, resource grants,
  deterministic weighted loot table rolls, dialogue logging, level transitions,
  level-local flags, and saved once-event state.
- Runtime path following for idle enemies and non-enemy path-bound props.
- Simple primary fire against enemy props, with solid world geometry blocking
  shots before enemy ray/sphere hit checks are applied.
- Baseline data-driven enemy chase/attack using enemy definition activation,
  movement, range, wind-up, damage, and cooldown stats.
- Enemy prop removal from level data, physics, and render instances.
- Prototype resource pickup, Anchor banking, unsecured resource loss on death,
  and local Anchor respawn.
- Hurtbox damage, death, delayed respawn, and player restoration.
- Procedural ambient and one-shot audio.
- Asset catalog scanning for model assets.
- Configurable console position/stamina debug logging.
- Strict startup validation for tuning, bindings, registries, level data, and
  base-map geometry with actionable errors instead of silent substitution.
- Non-window project content validation through `cargo run -- validate`.
- Whole-project diagnostics through `cargo doctor`.
- Transactional F5 reload for tuning, bindings, models, textures, enemy/relic
  definitions, and the current level; rejected changes leave live state intact.
- Validated staged writes for levels, prefabs, and saves with cross-process
  locking and interrupted-write recovery.

## Level Data Contract

`LevelData` lives in `src/data/world/level.rs`.

Required level fields:

- `name`
- `base_map`
- `player_spawn`
- `props`

Optional level authoring fields with safe defaults:

- `asset_imports`: `[]`
- `loot_tables`: `[]`
- `paths`: `[]`
- `events`: `[]`
- `dialogues`: `[]`

Required prop field:

- `asset_id`

Prop fields with safe defaults:

- `id`: `null`
- `position`: `[0.0, 0.0, 0.0]`
- `rotation`: `[0.0, 0.0, 0.0]` in degrees
- `scale`: `[1.0, 1.0, 1.0]`
- `collider_type`: `"None"`
- `is_climbable`: `false`
- `is_hurtbox`: `false`
- `item_id`: `null`
- `resource_value`: `0`
- `anchor_id`: `null`
- `enemy_type`: `null`
- `enemy_health`: `0.0` before runtime definition materialization
- `light_color`: `null`
- `light_intensity`: `0.0`
- `ambient_sound_id`: `null`
- `trigger_level_id`: `null`
- `loot_table_id`: `null`
- `path_id`: `null`
- `dialogue_id`: `null`
- `event_id`: `null`

Level validation currently checks:

- level name is non-empty
- base map exists
- player spawn is finite
- prop asset exists under `assets/`
- prop transforms are finite
- prop scale contains no zero values
- enemy props do not have negative health
- anchor IDs are not empty
- resource pickups are not also enemies
- level transition targets exist under `levels/`
- lit props have a color when intensity is set
- authoring IDs use stable ID characters and are unique per collection
- prop `loot_table_id`, `path_id`, `dialogue_id`, and `event_id` references
  resolve to level-local authoring data
- asset imports reference existing model assets and have valid defaults
- loot tables have rolls, entries, weights, quantities, and valid grant shapes
- paths have finite waypoints and valid speed multipliers
- events have valid trigger data and action requirements
- dialogues have speakers and non-empty lines

Project validation through `cargo run -- validate` also checks:

- enemy definition TOML files parse
- enemy definition IDs are unique
- enemy definition stats are finite and sane
- enemy definitions include a readable visual tell
- level `enemy_type` values match enemy definition IDs
- runtime level loading applies enemy definition model, collider, and health to
  enemy props before physics/render/combat use them
- `config/tuning.toml` parses into `GameConfig`
- numeric tuning values are finite and within sane ranges
- sprint speed is not lower than walk speed
- combat, world, and lighting values are usable
- `config/bindings.toml` contains required actions
- binding tokens are valid
- duplicate bindings are reported, except explicit unbound values
- all model assets under `assets/` use supported extensions
- all model assets under `assets/` can be loaded by the mesh loader
- all model assets under `assets/` contain vertices, render indices, and
  physics triangles
- supported texture files under `textures/` decode successfully

## Save Contract

Autosaves are validated before writing. The save system writes through a
flushed same-directory staging file, retains the previous valid state at
`save/cenotaph_save.backup.json`, and automatically repairs a missing or
damaged primary from that backup when `continue` is used. Save data rejects
unsafe level IDs, unsupported versions, non-finite positions, duplicate IDs,
and equipped relics that are not owned.

## Foundation Test Level

Run:

```powershell
cargo run -- foundation_test
```

`levels/foundation_test.json` contains one example of each current prop role:

- decorative cube
- resource pickup
- Anchor banking prop
- static box collider
- enemy sphere collider
- hurtbox sphere collider
- transition trigger to `ashwalk_01`

This level is intentionally plain. Its job is to make engine assumptions easy
to test before Ash-Walk content becomes more complex.

## Config Contract

`config/tuning.toml` now maps to:

- `PlayerConfig`
- `MovementConfig`
- `CameraConfig`
- `PhysicsConfig`
- `CombatConfig`
- `WorldConfig`
- `LightingConfig`
- `DebugConfig`

Newer sections use defaults when omitted, so older tuning files can still load.

Current combat foundation knobs include:

- `base_damage`
- `crit_multiplier`
- `attack_cooldown`
- `miss_cooldown`
- `primary_fire_range`
- `enemy_hit_radius`
- `hurtbox_damage_per_second`
- `hurtbox_radius`
- `hurtbox_tick_interval`
- `respawn_delay`

Current movement foundation knobs include:

- `dash_speed_multiplier`
- `dash_stamina_cost`
- `dash_cooldown`
- `dash_duration`
- `sprint_stamina_drain_rate`

Current debug foundation knobs include:

- `position_log_enabled`
- `position_log_interval`

## Enemy Definition Contract

Enemy definitions live in `data/enemies/*.toml`.

Required fields:

- `id`
- `display_name`
- `role`
- `behavior_tag`
- `model_asset`
- `collider_type`
- `visual_tell`
- `health`
- `damage`
- `move_speed`
- `activation_range`
- `attack_range`
- `attack_windup`
- `attack_cooldown`

The current starter definitions are Ashbound, Burdened, Censer, Chainrunner,
and Harpy. They point at authored low-poly silhouette models under
`assets/enemies/` until production enemy meshes are generated and hooked up.

Level files should treat `enemy_type` as the authored enemy selector. Runtime
loading fills the prop's model asset, collider, and health from the matching
enemy definition. The current baseline AI also reads enemy definition activation
range, movement speed, attack range, attack wind-up, damage, and attack
cooldown.

## Checks

Use this before treating the project as stable:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/project_check.ps1
```

The old `scripts/foundation_check.ps1` path delegates to the project check.

The script runs:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- doctor
```

Manual play smoke checks:

```powershell
cargo run -- play ashwalk_01
cargo run -- play foundation_test
```

Use `FOUNDATION_SMOKE_CHECKLIST.md` for the manual pass/fail launch checklist.

The automated test suite covers config defaults, key parsing, level defaults,
level validation, enemy definition validation, content validation reports, model
asset validation, asset catalog serialization, player health, stamina, dash, and
respawn state, resource banking, and unsecured resource loss.

## Enemy Documentation

The current runtime has prototype prop shooting plus baseline data-driven enemy
chase/attack. `levels/foundation_test.json` uses a `Burdened` enemy prop whose
runtime model, collider, health, and combat tuning come from
`data/enemies/burdened.toml`.

Gameplay-first enemy roles are documented in `../art/ENEMY_GAMEPLAY_ROSTER.txt`.
3D model generation guidance is documented in
`../art/ENEMY_MODEL_GENERATOR_BRIEF.txt`.

## Current Non-Goals

Do not treat these as foundation requirements yet:

- relic generation
- full inventory UI and item comparison
- advanced enemy AI, pathfinding, and role-specific behavior
- full Anchor UI, Anchor persistence, and Sanctuary behavior
- sanctuary UI
- save slots, migration, and conflict handling
- full run-contract Cycle Director
- procedural route generation
- full content registry
- native/DCC-style editor application beyond the browser localhost editor
- freeform mesh sculpting beyond prop/brush geometry

These belong after the foundation remains boringly reliable.

## Next Stable Targets

1. Add typed hazard data once hazards graduate beyond hurtbox props.
2. Add production enemy models and behavior-specific animation/audio tells
   before expanding the enemy roster in levels.
