# Solo Developer Critique

Honest assessment of what's making Cenotaph harder to build than it needs to be.
Written to inform refactoring priorities, not to negate the work already done.

---

## Executive Summary

The project has excellent design documentation, solid content validation, and a
clear identity. The problems are all on the code-organization side: a god object
that owns everything, a monolithic update loop, a universal prop struct that
forces 25-field constructions everywhere, and thousands of lines of custom engine
code that could be thinner. None of these are fatal. All of them compound — every
new feature costs more than it should because it has to navigate the tangles.

The single highest-leverage change is splitting `EngineState` and the update loop.
Everything else becomes easier once that's done.

---

## Critical Issues

### 1. EngineState Is a 47-Field God Object

**File:** `src/core/engine/state.rs` (2,266 lines)

`EngineState` owns the GPU device, window surface, render pipeline, all buffers,
all textures, the camera, the physics engine, the audio system, the particle
system, the HUD system, the entire level, all enemies, all player state, all
progression, the save system, feedback, dialogue, mountain reactions, and cycle
state. Every field is owned. Zero lifetimes. Zero references.

**Why this hurts:**
- You cannot test combat without initializing wgpu.
- You cannot test HUD rendering without loading a level.
- `new()` is 346 lines because it has to initialize everything in one function.
- Adding a single field to any subsystem means touching `state.rs`, then every
  method that constructs or reads that field.
- `render()` builds a 40-field `HudFrameState` struct inline with complex logic
  that should live on a HUD builder.

**The fix:**
Extract ownership into focused structs that `EngineState` composes rather than
flattening:

```
EngineState
  gpu: GpuContext          (device, queue, surface, pipeline, buffers)
  world: WorldState        (level, props, enemies, physics, paths)
  player: PlayerState      (already exists — promote to owned)
  progression: RunProgress (already exists — promote to owned)
  render: RenderState      (camera, lighting, particles, assets, textures)
  feedback: FeedbackState  (already exists — promote to owned)
  audio: AudioSystem       (already exists — promote to owned)
  hud: HudSystem           (already exists — promote to owned)
  save: SaveState          (save data, backup state)
```

Each sub-struct gets its own `new()`, `update()`, and `tick()` methods.
`EngineState` becomes a thin orchestrator that calls them in order. This is not
a rewrite — it's moving fields into the structs that already exist but are
currently held by reference or passed as parameters.

### 2. update_physics() Ticks 24 Systems in One 305-Line Function

**File:** `src/core/engine/update.rs` (2,299 lines)

The main simulation function `update_physics()` runs mountain reactions, particle
updates, action cooldowns, feedback, dialogue, debug input, anchor rites,
movement, sprint/dash/stamina, level transitions, interaction, event evaluation,
enemy AI, path following, physics stepping, landing detection, audio, prop sync,
progression, combat, respawn, hurtboxes, and debug logging — sequentially, with
early returns, nested if/else, and interleaved concerns.

**Why this hurts:**
- Adding a new system means finding the right insertion point in 305 lines.
- There's no way to know what depends on what without reading every line.
- Testing any one system requires the full engine state.
- Debugging order-dependent bugs is a nightmare.

**The fix:**
Split into named phases with clear ownership:

```
tick_movement(&mut self, dt)       — intent, sprint, dash, stamina
tick_world(&mut self, dt)          — transitions, interactions, events, props
tick_combat(&mut self, dt)         — fire, raycast, damage, kill, loot
tick_enemies(&mut self, dt)        — AI, path following, attacks
tick_persistence(&mut self, dt)    — respawn, hurtbox damage, progression
tick_atmosphere(&mut self, dt)     — mountain reactions, particles, audio, feedback
```

Each phase is a method on the relevant sub-struct, not on `EngineState`. The
update function becomes a 10-line orchestrator. This is already partially done —
`update_enemy_ai`, `handle_gameplay_input`, `update_anchor_rite` exist as
methods. They just need to be moved to the sub-structs that own their data.

### 3. PropData Is a 25-Field Universal God Struct

**File:** `src/data/world/level.rs`

Every object in the world — enemies, pickups, lights, anchors, hurtboxes,
triggers, dialogue sources, decorations — is a `PropData` with 25 fields. A
harmless rock carries `enemy_type: None`, `resource_value: 0`, `anchor_id: None`,
`loot_table_id: None`, etc. Construction is duplicated in 3 places because
there's no builder or constructor.

**Why this hurts:**
- Every `PropData` construction (tests, loot spawning, save restoration) must
  spell out 25 fields, most of which are irrelevant defaults.
- You can't tell from the struct what a prop *is* — it might be an enemy, a
  light, or a rock. The role is determined by which fields are `Some`.
- Adding a new prop role means adding more fields to the universal struct.
- Validation has to check "if this field is Some, those fields must also be Some"
  instead of checking typed role structs.

**The fix:**
Introduce `PropRole` as a tagged enum:

```rust
enum PropRole {
    Decoration,
    Enemy { enemy_type: String },
    Resource { resource_value: u32 },
    Anchor { anchor_id: String },
    Hurtbox { damage: f32 },
    Light { color: [f32; 3], intensity: f32 },
    Dialogue { dialogue_id: String },
    Trigger { event_id: String },
    Relic { relic_id: String },
}
```

`PropData` keeps the shared fields (transform, model, collider, visible) and
has one `role: PropRole` field. Construction becomes `PropData::newDecoration(...)`
or `PropData::newEnemy(...)`. Each role variant carries only the fields it needs.
The 3 construction sites collapse to 1 per role.

---

## High Issues

### 4. Custom Engine Tax

You maintain ~5,000+ lines of custom engine code:
- wgpu renderer: ~3,000 lines across 8 files
- Immediate-mode HUD with bitmap font: ~1,660 lines
- Rapier3D wrapper: ~300 lines
- Particle system: ~300 lines
- Audio system: ~200 lines

This is not wasted work — it works and it's yours. But it's a recurring cost.
Every new visual feature (shadows, post-processing, text rendering, UI widgets)
requires writing engine code instead of configuring a framework.

**Honest assessment:** Switching to Bevy or another framework at this point
would be a full rewrite of everything that exists. That's not worth it. But you
should be aware that every future rendering feature costs 10x what it would cost
in a framework. The pragmatic move is to keep the custom engine but be ruthless
about extracting reusable patterns from it — a widget system, a scene graph, a
material system — so new features compose rather than accumulate.

**Specific pain point:** The HUD bitmap font (`glyph_rows()` at 121 lines)
only supports A-Z, 0-9, and 3 punctuation characters. Any future localization,
dynamic text, or player-facing text will require replacing this. Consider
`glyphon` or `cosmic-text` integration now rather than later.

### 5. Level Schema Is Overloaded

**File:** `src/data/world/level.rs` (2,819 lines)

`LevelData` has 14 top-level fields and 13 nested struct types.
`LevelEventActionData` is a tagged union flattened into 8 optional fields where
only `kind` and one payload are meaningful. There are 18 default-value functions
just for serde deserialization.

**Why this hurts:**
- Authoring levels means understanding a 14-field top-level struct with deeply
  nested optionals.
- Adding a new event action means adding a field to `LevelEventActionData` and
  updating validation, even if the field is irrelevant to most events.
- The schema has grown organically — events, dialogue, paths, loot, terrain,
  and atmosphere all live in the same struct.

**The fix:**
Split `LevelData` into composable sections:

```
LevelData
  metadata: LevelMetadata     (id, name, stratum, version, base_map)
  atmosphere: AtmosphereData  (fog, wind, particles, lighting)
  terrain: TerrainData        (brush geometry, colliders)
  props: Vec<PropData>        (all placed objects)
  events: Vec<LevelEvent>     (triggers + actions)
  dialogue: Vec<DialogueLine>
  paths: Vec<LevelPath>
  loot: Vec<LootTable>
```

This doesn't change the JSON schema — it just organizes the Rust side. Each
section can have its own validation function. `LevelEventActionData` becomes a
proper tagged enum with serde support.

### 6. No Unit Tests for Game Logic

The project has tests in `state.rs` (~480 lines) and `update.rs` (~220 lines)
and `level.rs` (~675 lines), but they're mostly schema validation and
construction tests. There are zero tests for:
- Combat math
- Damage calculation
- Enemy AI behavior
- Player movement
- Resource pickup
- Anchor rite progression
- Save/load round-trips
- Level event evaluation

**Why this matters for a solo dev:** You can't manually test everything every
time you change something. A single combat math bug could go unnoticed for
weeks. The `GAMEPLAY_IDENTITY_RESET.md` lists many undefined design decisions —
when you do define them, you'll want tests to catch regressions.

**The fix:** After the EngineState split, game logic methods will take focused
struct references instead of `&mut self` on the whole engine. At that point,
unit tests become trivial to write because you can construct a minimal test
context without GPU init.

---

## Medium Issues

### 7. Documentation Has One Real Duplication

The 6,900-line doc corpus is mostly well-scoped. The one structural issue:
`CENOTAPH_STRATA.md` duplicates all 7 strata definitions from `CONTENT_GUIDE.md`
verbatim in condensed form. Two sources of truth for the same data.

**Fix:** Reduce `CENOTAPH_STRATA.md` to a pure index (name + one-line theme +
link to `CONTENT_GUIDE.md`). This cuts ~80 lines and removes the only real
multi-source-of-truth risk.

### 8. The Backlog Is Doing Too Much

`PROJECT_IMPROVEMENT_BACKLOG.md` has 158 lines covering reliability, iteration,
testing, architecture, tooling, player-facing features, and release — all in
one flat list. The items range from "add CI profiles" to "define death
consequences." These are not the same kind of work.

**Fix:** Split into:
- **Architecture refactors** (EngineState split, PropData builders, update phases)
- **Content gates** (undefined design decisions — these are blocked, not backlogged)
- **Polish tasks** (settings UI, loading screens, accessibility)
- **Tooling tasks** (dev console, loot simulator, navmesh)

### 9. `render()` Builds HUD State Inline

`EngineState::render()` constructs a ~40-field `HudFrameState` struct inline
(lines 729–768) with complex logic for dash cooldown, health/trail ratios, and
cycle modifiers. This is presentation logic living in the wrong place.

**Fix:** Move to `HudSystem::build_frame_state(&self, engine: &EngineState)`.
The HUD system already exists — it should own its state construction.

---

## What's Actually Good

Not everything is broken. These are worth preserving:

1. **Identity documentation hierarchy.** The IDENTITY_CONTRACT → GAMEPLAY_IDENTITY_RESET → everything-else chain is clean, non-contradictory, and prevents drift. This is rare in solo projects.

2. **Content validation.** `cargo validate` and `cargo doctor` catch real problems. The validation system is thorough and well-structured. Don't regress on this.

3. **Transactional hot-reload.** F5 reload that atomically swaps config, levels, registries, models, and textures is a genuine quality-of-life feature for a solo dev. This earns its complexity.

4. **Save system.** Staged writes with backup recovery and `SaveFileHealth` states. This is appropriately robust for an autosaving game. The main issue is duplicated `PropData` construction, not over-engineering.

5. **Data-driven content.** Enemies, relics, levels, config, visual profiles, and HUD colors are all in data files. Adding content doesn't require touching Rust code (unless adding new behavior). This is the right architecture.

6. **HUD decomposition.** Unlike the rest of the engine, the HUD is well-modularized into sub-widgets (anchor_rite, ascent, dialogue, encounter, markers, notifications, overlays, player_status). This pattern should be replicated elsewhere.

7. **Build performance.** 0.33s incremental builds. 14 lean dependencies. No build scripts or proc macros. This is a fast project to compile.

---

## Prioritized Refactoring Plan

### Phase 1: Untangle the God Object (Highest Impact)

**Goal:** `EngineState` goes from 47 fields to ~15. Every subsystem owns its state.

**Steps:**
1. Promote `PlayerState`, `RunProgress`, `FeedbackState`, `AudioSystem`,
   `HudSystem` to owned fields on `EngineState` (they already exist as types).
2. Extract `GpuContext` struct (device, queue, surface, pipeline, depth buffer,
   shader module). `EngineState::new()` calls `GpuContext::init()`.
3. Extract `WorldState` struct (level data, props, enemies, loot, paths, events,
   dialogue flags, mountain reactions). This is the mutable game world.
4. Extract `RenderState` struct (camera, lighting, particles, assets, textures,
   instance buffers). This is the rendering pipeline's local state.
5. Move `update_physics()` systems into methods on their owning sub-structs.
   `EngineState::tick()` becomes a 10-line orchestrator.

**Estimated effort:** 3–5 focused sessions. No API changes outside the engine.

**Verification:** All existing tests pass. `cargo validate` and `cargo doctor`
pass. Manual playtest of Ash-Walk.

### Phase 2: Fix PropData (Second Highest Impact)

**Goal:** `PropData` goes from 25 fields to ~8 shared + a role enum.

**Steps:**
1. Define `PropRole` enum with variants for each prop role.
2. Add `PropData::newDecoration()`, `newEnemy()`, `newResource()`, etc.
   constructors.
3. Replace all 3 construction sites with the appropriate constructor.
4. Update serde to handle `role` as a tagged enum.
5. Update validation to check role-specific invariants.

**Estimated effort:** 1–2 sessions.

**Verification:** All levels load. All props spawn correctly. Save/load round-trips.

### Phase 3: Level Schema Cleanup (Medium Impact)

**Goal:** `LevelData` becomes a thin container over composable sections.

**Steps:**
1. Group `LevelData` fields into section structs (metadata, atmosphere, terrain).
2. Convert `LevelEventActionData` from flat optional fields to a proper tagged
   enum.
3. Move validation into per-section methods.
4. Update JSON schema (version bump + migration).

**Estimated effort:** 1–2 sessions.

**Verification:** All levels load. Validation catches the same errors as before.

### Phase 4: Documentation Cleanup (Low Effort, Good Hygiene)

**Steps:**
1. Reduce `CENOTAPH_STRATA.md` to an index linking to `CONTENT_GUIDE.md`.
2. Split `PROJECT_IMPROVEMENT_BACKLOG.md` into categories.

**Estimated effort:** 1 session.

### Phase 5: Unit Tests for Game Logic (Medium Effort, High Confidence)

**Steps:** After Phase 1, game logic methods take focused struct references.
Write tests for:
- Combat math (damage calculation, ray-sphere intersection)
- Player state transitions (sprint, dash, death, respawn)
- Anchor rite progression
- Resource pickup and banking
- Level event evaluation
- Save/load round-trips

**Estimated effort:** 2–3 sessions.

---

## What to Stop Doing

1. **Stop adding fields to `PropData` for new features.** If a new prop role
   needs new data, add a `PropRole` variant instead.

2. **Stop writing methods on `EngineState` for gameplay logic.** If it modifies
   player state, it goes on `PlayerState`. If it modifies enemies, it goes on
   `WorldState` or `EnemyRuntimeState`.

3. **Stop building HUD state in `render()`.** The HUD system builds its own
   frame state.

4. **Stop adding documentation for systems that don't exist yet.** The
   TECHNICAL_NOTES.md lists 20+ future systems with detailed specs. Write those
   specs when you're about to implement them, not before.

5. **Stop treating the backlog as a single list.** Split it. The architecture
   refactors unlock everything else. Do those first.
