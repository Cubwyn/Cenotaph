# Cenotaph: The Great Omission

## Master Project README

---

## One-Sentence Vision

**Cenotaph is a surreal vertical looter RPG about surviving repeated ascents through a hostile mountain that changes, forgets, and rewards adaptation.**

---

## Project Identity

Cenotaph is not a movement shooter.

Cenotaph is not a pure roguelike.

Cenotaph is not a survival simulator.

Cenotaph is not a fixed-scope campaign RPG.

Cenotaph is a long-term modular game project built around:

- Vertical survival
- Relic-based loot
- Buildcrafting
- Replayable ascents
- Dreamlike world logic
- Oppressive atmosphere
- Expandable systems

The player climbs because staying below is impossible.

Loot is not the only goal.

Loot is memory, survival, temptation, and adaptation.

---

## Core Fantasy

The player is a pilgrim ascending an impossible mountain made from collapsed kingdoms, petrified cathedrals, broken machinery, hanging settlements, and forgotten realities.

The mountain is not simply a location.

The mountain is the central force of the game.

It resists, forgets, mutates, suppresses, and reveals.

The player should feel:

- Small
- Isolated
- Curious
- Pressured
- Determined
- Increasingly capable, but never fully safe

The game should feel like climbing through a dream that has rules, but refuses to explain them.

---

The core design question is:

```text
Can the player survive the next part of the climb?
```

Not only:

```text
Can the player find better loot?
```

---

## Development Philosophy

Cenotaph is intended as a forever project.

This does not mean every idea should be built immediately.

It means the foundation must allow new content to be added for years without breaking the project.

The project should be built as:

```text
Stable Core
+
Expandable Content
+
Dreamlike Presentation
```

The stable core must be small, reliable, and boringly solid.

The expandable content can be strange, ambitious, and endless.

Because Cenotaph is a solo project, the production model must lean on
programming leverage: reusable assets, data-driven content, generated support,
strict validation, strong atmosphere, and systemic variation. The game should
not require large volumes of bespoke modeling, animation, cinematics, or
hand-authored detail before the core ascent loop feels real.

---

## Golden Rules

1. The mountain is the protagonist.
2. Loot must support survival and buildcraft.
3. Movement must feel impactful, not trendy.
4. Replayability comes from recombination, not endless handcrafted content.
5. Dream logic must still have design logic.
6. Every system must plug into the core loop.
7. Add content freely. Add core systems cautiously.
8. The game must always have a playable version.
9. A feature is not real until it works inside an ascent.
10. If a feature does not improve the climb, survival, relic hunting, buildcraft, replayability, or mystery, cut it.
11. Prefer reusable systems and asset-efficient presentation over bespoke content volume.

---

## Documentation Map

This README is the master overview. The categorized documentation index lives
at [`docs/README.md`](docs/README.md):

- `docs/design/IDENTITY_CONTRACT.md` is the top-level identity guardrail for
  future design, content, and coding-agent work.
- `docs/design/ASH_WALK_PILGRIMAGE.md` is the current playable milestone and
  acceptance gate before adding another Stratum.
- `docs/design/` contains systems, lore, content rules, and the roadmap.
- `docs/development/` contains the current foundation, smoke checklist,
  technical notes, and ordered project-wide improvement backlog.
- `docs/art/` contains asset rules and enemy art/model briefs.
- `docs/archive/` preserves superseded project inventories for history only.

## Project Layout

```text
assets/         Runtime-ready models
config/         Player bindings and gameplay tuning
data/           Enemy and relic definitions
docs/           Current categorized documentation and historical archive
levels/         Runtime JSON levels only
prefabs/        Validated reusable prop groups for level construction
scripts/        Repeatable project checks
source_assets/  Blender files, conversion inputs, and visual references
src/            Rust runtime, data contracts, systems, and developer tooling
textures/       Runtime textures
```

Generated caches are ignored. Launching the game does not write generated
content into the level, prefab, asset, or texture directories.

---

## Current Foundation Build

The current codebase is a foundation prototype built on:

- Rust 2021
- winit
- wgpu
- Rapier3D
- rodio
- JSON level data
- Level-authored materials, fog, particles, wind, and procedural ambience
- Camera-aware fog/lighting, shader-driven prop motion, and bounded impact particles
- Grounded movement cadence plus distinct procedural movement and combat cues
- TOML tuning and bindings
- TOML enemy definitions
- TOML relic definitions

Run the default movement/combat sandbox:

```powershell
cargo run
```

Resume the autosave:

```powershell
cargo run -- continue
```

Run the current Ash-Walk test map:

```powershell
cargo run -- play ashwalk_01
```

Run the foundation systems smoke-test map:

```powershell
cargo run -- play foundation_test
```

List playable level IDs or show every project command:

```powershell
cargo levels
cargo run -- help
```

Validate authored level, prefab, config, enemy, relic, model, and texture data
without opening a game window:

```powershell
cargo run -- validate
```

Run the broader project diagnosis. This also checks required directories,
default-level availability, save/backup health, source/runtime separation,
pending write sidecars, and level prop/dynamic-instance/base-map triangle
budgets:

```powershell
cargo doctor
```

Regenerate the deterministic prototype texture kit without third-party Python
packages:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/generate_prototype_textures.ps1
```

Run the full project check:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/project_check.ps1
```

`scripts/foundation_check.ps1` remains as a compatibility entry point.

Fast project aliases:

```powershell
cargo dev-check          # Check every Rust target
cargo doctor             # Diagnose layout, content, saves, and pending writes
cargo levels             # List playable level IDs
cargo validate-content   # Validate levels and data without opening a window
cargo play-foundation    # Launch the foundation systems test level
cargo resume             # Resume the latest autosave
```

Autosaves are validated before writing, replaced through a flushed staging
file, and keep `save/cenotaph_save.backup.json` as the last known-good slot.
`continue` automatically recovers that backup when the primary is missing or
damaged; `cargo doctor` reports the condition without opening the game.

Then use `docs/development/FOUNDATION_SMOKE_CHECKLIST.md` for manual launch/play validation.

Or run the core checks individually:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doctor
```

Core controls:

- `WASD` move
- `Space` jump
- `Shift` sprint
- `Q` dash
- `E` interact or advance the active dialogue line
- mouse look
- left mouse primary fire
- `I` cycle owned relics
- `F1` toggle the smoothed FPS/frame-time and world-count overlay
- `F5` transactionally reload tuning, bindings, models, textures, enemy/relic
  definitions, and the current level; invalid changes leave play state intact
- `Escape` pause/unpause

Controlled level authoring workflow:

- build base-map geometry in a source DCC file, export it into `assets/`, and
  reference it with the level's `base_map` field
- make deliberate JSON changes under `levels/`, `prefabs/`, `data/`, and
  `config/`; the game has no content-writing or freeform generation surface
- use stable prop IDs and explicit asset imports, loot tables, paths, events,
  dialogue blocks, materials, and atmosphere settings
- run `cargo run -- validate` before launching or reloading a changed level
- use `F5` in play mode to transactionally reload valid content; rejected
  changes leave the current play state intact

The runtime can execute on-enter/proximity events, grant resources, spawn
weighted loot rolls, queue level transitions, present timed or advanceable
dialogue, persist level-local flags and once-event state, and move path-bound
enemies or props. Continue also reconstructs removed authored props and loose
generated loot instead of replaying or discarding the encounter state.

The in-game HUD keeps permanent chrome restrained and surfaces contextual
interaction prompts, dialogue, shot-result feedback, and the event feed only
when they matter. Health includes a delayed loss trail, the reticle reacts to
combat results, and short level-arrival titles establish place without blocking play.

See `docs/development/FOUNDATION.md` for the current runtime contracts and stable groundwork checklist.

---

## Master Systems

Cenotaph is built around five master systems:

1. **The Ascent System** — strata, routes, anchors, sanctuary, survival pressure, death, banking.
2. **The Relic System** — weapons, loot, rarity, affixes, named relics, inventory.
3. **The Build System** — stats, perks, synergies, statuses, risk/reward effects.
4. **The Threat System** — enemies, modifiers, elites, bosses, hazards, suppression.
5. **The Cycle Director** — run contracts, mutations, replayability, persistent memory.

Supporting systems:

- Data Registry
- Content Validation
- Save System
- Presentation System
- Developer Tools

See `docs/design/SYSTEMS.md` for the full breakdown.

---

## World Structure

The world is divided into seven major Strata:

1. Ash-Walk
2. Ward of Irons
3. Hanging Slums
4. Sanctuary
5. Gallery of Wind
6. Mirror-Crust
7. The Breach

Each Stratum should be documented uniformly using the template in `docs/design/CONTENT_GUIDE.md`.

---

## Replayability Principle

Replayability is a first-class design pillar.

Cenotaph should not rely on endless handcrafted content.

It should rely on systems that recombine into new problems.

```text
Replayability
=
Relic Difference
+
Build Difference
+
Threat Difference
+
Route Difference
+
World Difference
+
Memory Difference
```

Each ascent should change at least three things:

```text
What the player finds
What threatens the player
What the mountain does
```

---

## Feature Filter

Before adding a feature, ask:

1. Does it improve the climb?
2. Does it improve survival?
3. Does it improve relic hunting?
4. Does it improve buildcraft?
5. Does it improve replayability?
6. Does it strengthen the mystery of the mountain?

If the answer is no to all six, the feature does not belong.

---

## Current North Star

The first major milestone is:

```text
First Ascent Prototype
```

Do not move to full Ash-Walk production until the First Ascent Prototype is fun.

Do not move to multiple Strata until Ash-Walk is fun.

Do not build forever content until the core can survive expansion.

---

## Current Limits

The current build now contains a test-protected First Ascent candidate, but it
remains a foundation prototype until the Ashwalk route, combat, and reward loop
pass the manual smoke checklist and feel worth replaying.

Implemented now:

- movement, sprint, dash, pause, readable guided HUD, audio feedback, level
  loading, config loading,
  hurtbox damage, death/respawn, prototype prop shooting with solid-world
  obstruction, baseline data-driven enemy chase/attack, prototype resource
  pickup, explicit Anchor rites, authored relic pickups, stable-source enemy relic
  rewards, named encounter/reward HUD, world-state autosave/resume, basic cycle modifiers, and
  full project content validation; Ashwalk now composes those systems into a
  compact readable climb with one hazard, paced resources, a relic choice, an
  authored named Ash-Warden and relic drop, a summit rite, reusable mountain
  reactions, and a return gate

Not implemented yet:

- advanced enemy AI, role-specific enemy behavior, production enemy models,
  generated relic rolls/affixes, item comparison UI, data-authored Anchor rite
  variants and full Sanctuary UI,
  a data-driven elite modifier, upgrade/perk choice, full run-contract Cycle
  Director, and procedural route generation

The deterministic prototype model kit can be regenerated without Blender or
third-party Python packages:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/generate_prototype_models.ps1
```


---

## Final Project Statement

Cenotaph is a forever project about building a playable mountain of memory, pressure, relics, and impossible ascents.

The goal is not to finish every idea.

The goal is to build a foundation strong enough that strange ideas can be added forever.

The player climbs because the world below cannot hold.

The mountain forgets.

The player remembers.

The relics remember incorrectly.

And every ascent becomes another version of the same impossible dream.

---
A solo project by Cubwyn
