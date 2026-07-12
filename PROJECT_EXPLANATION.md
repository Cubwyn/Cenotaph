# Cenotaph Project Explanation

This document explains how the current Cenotaph project works, what each major
part does, and how the pieces fit together.

It describes the project as it exists now, not the full future design. The
future vision is much larger than the current prototype.

## 1. What This Project Is

Cenotaph: The Great Omission is a Rust game prototype for a surreal first-person
vertical looter RPG about repeated ascents through a hostile mountain.

The long-term design is about:

- climbing upward through dangerous strata
- surviving pressure between checkpoints
- finding relics and changing builds
- replaying ascents that change between cycles
- keeping the mountain strange, oppressive, and readable

The current codebase is the foundation prototype. It proves that the project can
launch, load authored content, move the player, render a level, run physics,
handle basic combat, damage the player, respawn, and validate content.

The current prototype is not yet the full First Ascent Prototype. It now has
authored relic pickups/rewards, owned-relic cycling, autosave/resume, Anchor
banking, and basic cycle modifiers. Generated relic rolls, full inventory UI,
procedural routes, advanced enemy roles, and the full run-contract Cycle
Director are still future work.

## 2. Current Implemented Game Loop

The playable foundation loop is:

```text
start the game
load a level
spawn the player
move, jump, sprint, dash, and look around
interact with simple props
pick up unsecured resource
bank resource at an Anchor
fight baseline enemy props
take damage from enemies or hurtboxes
die and respawn after a delay
transition to another level through trigger props
```

The project has two important runtime levels:

- `ashwalk_01`: the current Ash-Walk shell, using `assets/ashwalk_001.obj`.
- `foundation_test`: the systems test arena for props, resource, Anchor,
  enemy, hurtbox, static collider, and level transition behavior.

## 3. How To Run It

Run the Ash-Walk map:

```powershell
cargo run -- ashwalk_01
```

Run the foundation systems arena:

```powershell
cargo run -- foundation_test
```

Validate content without opening a window:

```powershell
cargo run -- validate
```

Run the full automated foundation check:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/foundation_check.ps1
```

Core controls are loaded from `config/bindings.toml`:

- `WASD`: move
- `Space`: jump
- `Shift`: sprint
- `Q`: dash
- mouse: look
- left mouse: primary fire
- `Escape`: pause and unpause

## 4. Runtime Flow

At a high level, the runtime flow is:

```text
src/main.rs
  creates the app or runs validation

src/app.rs
  owns the winit event loop
  creates the window
  sends input/window events to the engine

src/core/engine/state.rs
  builds EngineState
  owns GPU, physics, audio, loaded content, player state, and render state

src/core/engine/update.rs
  runs per-frame gameplay and physics

src/core/engine/state.rs render()
  draws the 3D world and HUD
```

Each redraw frame does this:

```text
read input
compute capped delta time
update player timers, stamina, sprint, dash
check pickups, Anchors, transitions, enemies, combat, hurtboxes, death
apply movement to Rapier physics
step physics
sync moved enemy props back into level/render data
update camera and lighting
draw map, props, and HUD
```

## 5. Core Architecture

The source tree is split into four main responsibilities.

### `src/core`

Engine orchestration. This layer owns the running game state and wires the
other layers together.

Important files:

- `src/core/engine/state.rs`: the main runtime owner.
- `src/core/engine/update.rs`: per-frame gameplay and physics bridge.
- `src/core/engine/loader.rs`: loads textures and model assets.
- `src/core/engine/sync.rs`: rebuilds GPU instance buffers from level props.
- `src/core/engine/validation.rs`: checks content/config/assets without a game window.
- `src/core/engine/asset_catalog.rs`: scans `assets/` and can write `assets/props.json`.

### `src/data`

Plain data definitions and loading. This layer should stay mostly logic-free.

Important files:

- `src/data/world/level.rs`: JSON level format, prop format, collider types,
  defaults, and validation.
- `src/data/config/gameplay.rs`: TOML tuning structs and keybinding parsing.
- `src/data/enemy.rs`: TOML enemy definitions and enemy registry lookup.

### `src/game`

Gameplay state and rules that are not tied directly to GPU or OS APIs.

Important files:

- `src/game/player.rs`: health, stamina, sprint/dash state, hit flash, death,
  and respawn restoration.
- `src/game/progression.rs`: unsecured resource, banked resource, active Anchor,
  and Anchor respawn position.
- `src/game/enemy.rs`: baseline enemy intent and attack windup/cooldown logic.
- `src/game/combat.rs`: prototype ray-vs-sphere primary-fire hit test.

### `src/systems`

Lower-level engine systems.

Important files:

- `src/systems/input/manager.rs`: keyboard, mouse button, mouse motion, and
  scroll capture.
- `src/systems/physics/engine.rs`: Rapier3D physics wrapper, player movement,
  ground checks, jumping, prop bodies, and enemy prop velocities.
- `src/systems/audio/mod.rs`: rodio-backed procedural ambient audio and sound
  effects.
- `src/systems/render/*`: wgpu rendering, camera, mesh loading, instancing,
  textures, lighting, HUD, and shaders.

## 6. EngineState

`EngineState` is the central runtime object.

It owns:

- the `winit` window
- the `wgpu` surface, device, queue, depth texture, and render pipeline
- the loaded map mesh and map instance buffer
- the prop asset manager and texture manager
- active prop draw groups for instanced rendering
- the camera, camera uniform buffer, and camera bind group
- lighting and fog uniforms
- HUD rendering
- optional audio
- the Rapier physics engine
- loaded `GameConfig`
- loaded `EnemyRegistry`
- loaded `LevelData`
- `PlayerState`
- `RunProgress`
- enemy runtime timers aligned with level props
- pending level transitions

This is the file where startup construction happens. It loads config, loads
enemy definitions, loads the requested level, materializes enemy props from
enemy TOML, loads map geometry, builds GPU buffers, builds physics, starts
audio, creates player/progression state, and syncs props into render instances.

## 7. Data-Driven Content

The project is intentionally data-driven.

### Config

`config/tuning.toml` controls:

- player health and stamina
- walk and sprint speeds
- dash cost, cooldown, duration, and speed multiplier
- camera sensitivity
- gravity, jump velocity, and physics player speed
- combat damage, cooldowns, hit radius, hurtbox damage, and respawn delay
- fog and lighting values
- debug position logging

`config/bindings.toml` controls action bindings.

### Levels

`levels/*.json` files define playable spaces.

A level has:

- `name`
- `base_map`
- `player_spawn`
- `props`

A prop can represent many things depending on metadata:

- decoration
- physics collider
- resource pickup
- Anchor
- enemy
- hurtbox
- light marker
- ambient sound marker
- level transition trigger

The current prop format is intentionally broad so the foundation can test many
systems without needing separate registries for every content type yet.

### Enemies

`data/enemies/*.toml` defines enemy archetypes.

Each enemy definition includes:

- id and display name
- role and behavior tag
- model asset
- collider type
- readable visual tell
- health
- damage
- movement speed
- activation range
- attack range
- attack windup
- attack cooldown

When a level prop has `enemy_type = "Burdened"` or another enemy ID, runtime
loading fills in the prop model, collider, and health from the matching enemy
definition. The current baseline AI also reads movement and attack values from
that enemy data.

## 8. Physics

Physics uses Rapier3D.

The player is a dynamic rigid body with a sphere collider. Rotations are locked
so the player does not tumble. Movement is applied by setting horizontal linear
velocity based on camera-relative input.

The map uses triangle mesh collision when mesh data is available. A safety floor
is also created below the map to catch the player if they slip through large
triangles or missing collision.

Props are inserted into physics from level JSON:

- `None`: body exists but no collider
- `Box`: cuboid collider
- `Sphere`: ball collider
- `Mesh`: triangle mesh collider if mesh data exists

Enemy props are dynamic bodies so the AI can set their horizontal velocity.
Non-enemy props are fixed bodies.

Jumping is edge-triggered and supports a small coyote-time window, meaning the
player can still jump briefly after leaving the ground.

## 9. Rendering

Rendering uses `wgpu`.

The main renderer draws:

1. the base map
2. all active prop draw groups
3. the HUD overlay

Models are loaded from `.obj`, `.glb`, or `.gltf` files. The mesh loader returns
both render data and physics geometry.

Props are rendered with instancing:

```text
level props
  grouped by asset_id
  converted into per-instance transform matrices
  uploaded to GPU instance buffers
  drawn as batches
```

Textures are loaded from `textures/` when present. Missing textures use a
fallback checker texture, so broken or untextured assets still render visibly
instead of crashing.

The main shader handles:

- camera projection
- instanced transforms
- diffuse lighting
- attenuation
- rim/detail effects
- fog
- subtle color grading

The HUD shader is separate and draws simple screen-space colored quads for:

- health bar
- stamina bar
- crosshair
- pause overlay

## 10. Input

Input is handled by `InputManager`.

It records:

- currently pressed keyboard keys
- accumulated mouse movement for the frame
- scroll amount
- primary fire state

`app.rs` feeds winit events into the input manager. Escape is deliberately not
fully consumed by the input manager because `app.rs` uses it to toggle pause,
cursor capture, and ambient audio pause/resume.

## 11. Audio

Audio uses `rodio`.

There are no required audio files right now. The audio system procedurally
generates:

- ambient drone
- footstep
- hit
- pickup
- death sting
- level transition

Ambient starts on launch, pauses when the game pauses, and resumes when play
continues. Footsteps are currently triggered when movement crosses a threshold,
so they are a foundation cue rather than a full footstep timing system.

## 12. Combat And Enemies

Combat is intentionally minimal right now.

Primary fire casts a ray from the camera and checks whether it passes within
`combat.enemy_hit_radius` of an enemy prop position. If it hits, the enemy prop
loses `combat.base_damage`. When health reaches zero, the prop is removed from:

- level data
- physics
- render instances
- enemy runtime state

Enemy AI is baseline chase-melee logic:

- idle if the player is outside activation range
- chase if inside activation range but outside attack range
- attack if inside attack range

Attack timing uses windup and cooldown values from enemy TOML.

The enemy design docs describe many future roles, but those role-specific
behaviors are not implemented yet. At runtime today, all enemies use the same
basic activation/chase/attack structure, just with different stats.

## 13. Progression Foundation

`RunProgress` tracks the current lightweight progression loop:

- unsecured resource collected during the run
- banked resource saved at Anchors
- active Anchor ID
- active Anchor respawn position

Resource pickups are props with `resource_value > 0`. Walking near one collects
it and removes that prop.

Anchors are props with `anchor_id`. Walking near one activates it, banks all
unsecured resource, and sets the respawn position.

On death, the player loses only unsecured resource. Banked resource remains.

This is the foundation for the future ascent/banking loop, not a full economy
or save system yet.

## 14. Death And Respawn

The player can take damage from:

- hurtbox props
- enemy attacks

When health reaches zero:

- unsecured resource is lost
- player enters dead state
- dash/sprint state is cleared
- death sound plays
- respawn timer starts

After the respawn delay:

- health and stamina are restored
- the player body is moved to the active Anchor position, if any
- otherwise the player respawns at the level spawn

## 15. Validation

Validation is a major part of the project.

`cargo run -- validate` checks content without opening a game window.

It validates:

- level JSON parsing
- level required fields
- base map existence
- prop asset references
- finite transforms
- non-zero scale
- enemy health sanity
- empty enemy/Anchor IDs
- resource pickup and enemy conflicts
- transition target level existence
- light color/intensity consistency
- enemy TOML parsing
- duplicate enemy IDs
- enemy stat ranges
- enemy visual tells
- level enemy_type references
- tuning TOML parsing and value ranges
- binding TOML required actions, key tokens, and duplicate bindings
- all model asset extension support
- all model asset loadability
- model render and physics geometry

The tests also cover many of these contracts directly.

## 16. Project Folders

### `config/`

Designer-facing settings:

- `tuning.toml`: gameplay constants
- `bindings.toml`: keybindings

### `data/`

Structured gameplay definitions:

- `data/enemies/*.toml`: current enemy archetypes

### `levels/`

Level definitions and source/export files:

- `ashwalk_01.json`: Ash-Walk map shell
- `foundation_test.json`: foundation test arena
- `.blend`, `.obj`, `.mtl`: level art/source assets

### `assets/`

Runtime model assets:

- base maps
- cube placeholder
- enemy placeholder silhouettes
- generated asset catalog snapshot

### `textures/`

Optional PNG/JPG/JPEG textures. If none are present or a texture is missing,
the renderer uses the fallback checker texture.

### `scripts/`

Automation:

- `foundation_check.ps1`: runs format, clippy, tests, and validation

### `Dev/`

Reference/concept art.

### `src/`

Rust source code.

## 17. Design Documents

The root docs divide responsibilities well:

- `README.md`: master overview and command quickstart
- `FOUNDATION.md`: current stable foundation contract
- `FOUNDATION_SMOKE_CHECKLIST.md`: manual launch/play checklist
- `SYSTEMS.md`: long-term gameplay systems
- `CONTENT_GUIDE.md`: content templates and expansion rules
- `LORE.md`: tone, mythology, and writing rules
- `ROADMAP.md`: staged development plan
- `TECHNICAL_NOTES.md`: architecture and technical backlog
- `ASSET_GUIDE.md`: model asset rules
- `MODEL_RESET_PLAN.md`: planned model rebuild sequence
- `ENEMY_GAMEPLAY_ROSTER.txt`: enemy roles and encounter recipes
- `ENEMY_MODEL_GENERATOR_BRIEF.txt`: enemy model direction
- `layout_guide.txt`: source layout and dependency rules
- `PROJECT_DOCUMENTATION.txt`: practical file inventory
- `PROJECT_LAYOUT_FLOWCHART.md`: flowcharts and common paths
- `readme.txt`: short summary

## 18. Current Feature Status

Implemented:

- Rust 2021 single-crate app
- winit event loop
- wgpu renderer
- Rapier3D physics
- rodio procedural audio
- level JSON loading
- config TOML loading
- enemy TOML loading
- content validation
- first-person camera
- movement, jump, sprint, dash
- pause and cursor release
- HUD health/stamina/crosshair/pause overlay
- static prop collision
- hurtbox damage
- player death and delayed respawn
- resource pickup
- Anchor banking and local respawn
- level transition trigger
- prototype primary fire
- baseline enemy chase/attack
- model/texture fallback behavior

Not implemented yet:

- full weapon system
- generated relic rolls and affixes
- full inventory UI and item comparison
- save slots, migration, and conflict handling
- full Anchor UI
- Sanctuary UI
- advanced enemy AI
- role-specific enemy behavior
- production enemy models
- boss fights
- procedural route generation
- full run-contract Cycle Director
- full content registry
- full economy

## 19. Where To Add Things

Add or tune player movement/combat numbers:

```text
config/tuning.toml
src/data/config/gameplay.rs if a new field is needed
```

Add a new level prop:

```text
add model under assets/
add prop entry to levels/*.json
run cargo run -- validate
```

Add a new enemy archetype:

```text
add model under assets/enemies/
add data/enemies/name.toml
reference enemy_type from level JSON
run cargo run -- validate
```

Add new runtime enemy behavior:

```text
src/game/enemy.rs
src/core/engine/update.rs
```

Add new player state:

```text
src/game/player.rs
```

Add new run/resource/checkpoint state:

```text
src/game/progression.rs
```

Add new level/config validation:

```text
src/data/world/level.rs
src/data/config/gameplay.rs
src/core/engine/validation.rs
```

Add rendering behavior:

```text
src/systems/render/
src/core/engine/state.rs
src/core/engine/sync.rs
```

Add physics behavior:

```text
src/systems/physics/engine.rs
```

## 20. The Big Picture

The current project is a solid foundation prototype for a larger game. The code
already has the important skeleton:

```text
data files define content
validation protects content
EngineState loads and owns runtime state
systems handle low-level rendering/physics/input/audio
game modules hold reusable gameplay rules
the update loop turns input and data into playable behavior
```

The next natural step is not to build every future system at once. The roadmap
points toward the First Ascent Prototype: one small repeatable climb with route
choice, an Anchor, one hazard, one enemy encounter, one loot/relic stub, and a
reason to replay.
