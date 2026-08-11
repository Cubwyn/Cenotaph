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
- Relic definition validation from `data/relics/*.toml`, including the model
  used by each in-world pickup.
- Validation-enforced single-primitive placeholder assets under
  `assets/enemies/`, `assets/pickups/`, `assets/props/`, and `assets/world/`.
- First-person movement with jump, coyote time, sprint stamina, and dash.
- Pause mode through Escape, including cursor release and ambient pause/resume.
- Compact peripheral HUD with health loss trail, stamina and dash state,
  contextual interaction prompt, timed dialogue with interact-to-advance,
  responsive reticle, fading event feed, level-arrival title, pause overlay,
  and an opt-in F1 frame diagnostics panel.
- Deliberate, read-only runtime content loading: level and prefab files are
  changed explicitly in source control and never written by the game.
- Static, sphere, mesh, hurtbox, enemy, decorative, trigger, path-bound, and
  dialogue/event-linked prop data.
- Prop physics applies the same authored degree rotation as rendering, and mesh
  colliders apply authored scale.
- Level authoring graph data for asset imports, loot tables, paths, events, and
  dialogue blocks.
- Runtime level events for on-enter, proximity, nearest-prop interaction, and
  explicitly queued manual triggers; resource grants, deterministic weighted
  loot table rolls, in-world dialogue presentation, level transitions,
  level-local flags, queued mountain reactions, and saved once-event state.
- Runtime path following for idle enemies and non-enemy path-bound props.
- Simple primary fire against enemy props, with solid world geometry blocking
  shots before enemy ray/sphere hit checks are applied.
- Baseline data-driven enemy chase/attack using enemy definition activation,
  movement, range, wind-up, damage, and cooldown stats.
- Enemy health presentation preserves each prop's materialized maximum health,
  including authored elite overrides, and enemy markers appear only for active
  threats.
- Enemy prop removal from level data, physics, and render instances, including
  physical drops from an authored enemy `loot_table_id`.
- Save/resume world reconstruction for removed authored props and uncollected
  generated loot, including unfinished mountain-reaction queues. Event-linked
  defeats and Anchor claims commit mechanics and authored consequences
  atomically.
- Prototype resource pickup, Anchor banking, unsecured resource loss on death,
  and local Anchor respawn.
- Hurtbox damage, death, delayed respawn, and player restoration.
- Non-tonal filtered wind/pressure ambience. One-shot cue hooks remain wired but
  silent until authored recordings replace the removed synthesized tones.
- One-draw ambient and transient particle rendering with fixed 512/256 budgets,
  preset-specific shapes, gusts, edge fading, and gameplay bursts.
- Camera-aware fog, correct view rim lighting, shader-driven enemy/pickup/anchor
  motion, role-tinted materials, generated OBJ normals/UVs, and low-cost model
  material defaults from the generated Cenotaph texture kit.
- Dynamic prop transforms stream into existing GPU buffers; buffers are rebuilt
  only when the instance grouping or count changes.
- FIFO presentation is selected when available for predictable frame pacing.
- Configurable console position/stamina debug logging.
- Strict startup validation for tuning, bindings, registries, level data, and
  base-map geometry with actionable errors instead of silent substitution.
- Non-window project content validation through `cargo run -- validate`.
- Whole-project diagnostics through `cargo doctor`, including static prop,
  moving-instance, and base-map triangle budgets.
- Transactional F5 reload for tuning, bindings, models, textures, enemy/relic
  definitions, and the current level; rejected changes leave live state intact.
- Validated staged save-game writes with cross-process locking, backup
  recovery, and interrupted-write recovery.

## Level Data Contract

`LevelData` lives in `src/data/world/level.rs`.

Required level fields:

- `version` (current authored version: `1`)
- `name`
- `base_map`
- `player_spawn`
- `props`

Legacy files without `version` are treated as version `0` and migrated through
the shared level loader. Files newer than the current version are rejected
before validation or play.

Optional level authoring fields with safe defaults:

- `atmosphere`: clear/fog/key-light colors, bounded fog density, one instanced
  particle preset (None/Ashfall/Embers/Dust), particle budget and motion, wind,
  and non-tonal placeholder ambience preset/volume
- `base_material`: optional texture path relative to `textures/`, tint, UV
  tiling, and emissive strength
- `mountain_reactions`: reusable, validated atmosphere-response profiles
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
- `surface_material`: `null`; when present it can override texture, tint, UV
  tiling, and emissive strength
- `is_climbable`: `false`
- `is_hurtbox`: `false`
- `item_id`: `null`
- `resource_value`: `0`
- `anchor_id`: `null`; when present it is the unique persistence identity of an
  Anchor in this level
- `enemy_type`: `null`
- `enemy_health`: `0.0` uses the enemy definition; a positive authored value is
  preserved as an intentional per-instance override
- `light_color`: `null`
- `light_intensity`: `0.0`
- `ambient_sound_id`: `null`
- `trigger_level_id`: `null`
- `loot_table_id`: `null`
- `path_id`: `null`
- `dialogue_id`: `null`
- `event_id`: `null`; when present it must reference a `Manual` consequence
  fired by an enemy defeat or an Anchor's first binding

Level validation currently checks:

- level name is non-empty
- base map exists
- player spawn is finite
- atmosphere colors and numeric controls are finite and bounded
- particle counts do not exceed the fixed 512-instance runtime buffer
- base/prop material texture paths are safe, supported, and present under
  `textures/`
- prop asset exists under `assets/`
- prop transforms are finite
- prop scale contains no zero values
- enemy prop health is finite and non-negative
- anchor IDs use authoring-safe characters and are unique within the level
- resource pickups are not also enemies
- Anchor props cannot also be enemies or pickups
- level transition targets exist under `levels/`
- lit props have a color when intensity is set
- authoring IDs use stable ID characters and are unique per collection
- authored prop IDs cannot use the reserved `runtime_loot_` namespace; props
  linked to loot tables or events require stable IDs
- prop `loot_table_id`, `path_id`, `dialogue_id`, and `event_id` references
  resolve to level-local authoring data
- prop `event_id` hooks are limited to enemy or Anchor lifecycle consequences
  and reference `Manual` events; ordinary interaction uses an event trigger's
  `prop_id`
- asset imports reference existing model assets and have valid defaults
- loot tables have rolls, entries, weights, quantities, and valid grant shapes
- paths have finite waypoints and valid speed multipliers
- events have valid trigger data and action requirements
- repeatable automatic `OnEnter` and `Proximity` events are rejected because
  they would otherwise execute every frame; repeatable events must be explicit
  `Interact` or `Manual` triggers
- mountain reactions have unique IDs, valid colors and wind, positive duration,
  and finite atmosphere multipliers
- `ReactMountain` actions reference a declared reaction profile
- dialogues have speakers and non-empty lines

Project validation through `cargo run -- validate` also checks:

- enemy definition TOML files parse
- enemy definition IDs are unique
- enemy definition stats are finite and sane
- enemy definitions include a readable visual tell
- level `enemy_type` values match enemy definition IDs
- runtime level loading applies enemy definition model, collider, and health to
  enemy props before physics/render/combat use them
- relic pickup model paths are safe, present under `assets/`, and owned by the
  relic definition rather than a Rust lookup table
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

Continue reconciles the journal against current authored content before play.
Obsolete event, flag, reaction, relic, and removed-prop references are discarded
and the cleaned state is autosaved. Removed-prop records apply only to current
enemy or pickup props; they can never erase static world geometry or an Anchor.
An active Anchor is restored by stable `anchor_id`, with its current authored
position replacing stale saved coordinates. If that Anchor no longer exists,
the binding is cleared and the level spawn is used.

Stable authored IDs are persistence identities. Once shipped or used by a save,
an event, prop, flag, reaction, or Anchor ID must not be repurposed for different
content. Rename or retire an ID instead of recycling its meaning.

Only fired one-shot events are journaled. Repeatable events remain repeatable
after continue. Mountain-reaction queues are ordered and may contain the same
reusable profile more than once when separate authored consequences request it.

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

Current world/Anchor knobs include:

- `draw_distance`
- `fog_density`
- `anchor_interaction_radius`
- `anchor_mend_cost`

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
and Harpy. They point at neutral single-primitive placeholders under
`assets/enemies/`. Those proportions are non-canonical debugging scaffolding;
replacement requires developer-supplied visual direction.

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
level validation, enemy definition validation, content validation reports,
model asset validation and missing-normal generation, dialogue timing and
advancement, content budgets, player health, stamina, dash, respawn state,
Anchor rite selection, resource binding/spending, mountain reaction envelopes,
and unsecured resource loss.

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
- data-authored Anchor rite variants, Anchor world-state persistence, and
  Sanctuary behavior
- sanctuary UI
- save slots, migration, and conflict handling
- full campaign replayability/NG+ design (prototype Cycle modifiers are not a
  final progression contract)
- procedural route generation
- full content registry
- integrated visual level-authoring application
- freeform mesh sculpting beyond prop/brush geometry

These belong after the foundation remains boringly reliable.

## Next Stable Targets

1. Add typed hazard data once hazards graduate beyond hurtbox props.
2. Add separate collision-proxy assets and measured frustum/distance culling
   before growing level geometry or prop counts.
3. Move gameplay physics to a bounded fixed-step schedule.
4. Add production enemy models and behavior-specific animation/audio tells
   before expanding the enemy roster in levels.
