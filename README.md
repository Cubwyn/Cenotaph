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

---

## Documentation Map

This README is the master overview. The categorized documentation index lives
at [`docs/README.md`](docs/README.md):

- `docs/design/` contains systems, lore, content rules, and the roadmap.
- `docs/development/` contains the current foundation, smoke checklist,
  technical notes, and ordered project-wide improvement backlog.
- `docs/editor/` contains the standalone editor backlog.
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
tools/          Standalone editor web UI
```

Generated caches and editor backups are ignored. Launching the game no longer
writes an asset catalog into `assets/`.

---

## Current Foundation Build

The current codebase is a foundation prototype built on:

- Rust 2021
- winit
- wgpu
- Rapier3D
- rodio
- JSON level data
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
pending write sidecars, and retained editor backups:

```powershell
cargo doctor
```

Run the standalone level editor:

```powershell
cargo run -- editor
# Short project alias:
cargo editor
```

Then open the printed localhost URL. The editor lists real project levels,
assets, enemies, and relics; edits `levels/<level>.json`; validates through the
same Rust level contract as the game; and saves through the shared staged-write
system so the running game can hot-reload clean changes.

Security model: the editor binds only to `127.0.0.1`, prints a per-launch
tokenized URL, requires that token on all `/api/*` calls, rejects unexpected
`Host`/`Origin` headers, never serves arbitrary files, only writes safe
`levels/<id>.json` and `prefabs/<id>.json` paths, and creates ignored backup
copies before overwriting existing content or deleting a prefab.

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
- mouse look
- left mouse primary fire
- `I` cycle owned relics
- `F5` transactionally reload tuning, bindings, models, textures, enemy/relic
  definitions, and the current level; invalid changes leave play state intact
- `Escape` pause/unpause

Standalone editor workflow:

- choose a level from the Levels panel
- use the Hammer-style `4 View` layout: Camera, Top X/Z, Front X/Y, and Side
  Z/Y panels
- use the Camera viewport to select, place, or move props with ray picking
- use the Top/Front/Side panels to select, place, move, pan, zoom, and draw on
  exact orthographic grids; `Top Y`, `Front Z`, and `Side X` set each work plane
- fly the FPS-style editor camera with `WASD`/`QE`, right-drag to look, and
  wheel to move forward/back
- use the Keys workspace to inspect or remap editor keybindings; bindings are
  stored locally in the browser
- right-click Camera or orthographic views for context commands: select, place,
  create brush, paste, duplicate, focus, delete, validate, and reset camera
- use the Draw tool in Camera, Top, Front, or Side to create floor, wall, block,
  directional slope, configurable cylinder, directional stair, and closed
  terrain brushes
- select terrain geometry to raise/lower its center, smooth, flatten,
  regenerate, or generate a new seeded heightfield while preserving collision
- Ctrl-click or Shift-click the object list for multi-selection, or drag a
  marquee in an orthographic Select view; group operations preserve relative
  placement and show primary/secondary selection outlines
- use the XYZ axis selector, Snap toggle, grid value, transform inspector,
  arrow/PageUp/PageDown nudges, and Move tool for precise group transforms
- use the Palette for geometry, pickups, enemies, Anchors, hazards, and gates
- use the Asset Browser to place runtime-supported models or stage files from
  `source_assets/` for models, textures, materials, audio, dialogue/data,
  levels, and config
- use the dedicated Props, Assets, Events, Loot, Paths, Dialogue, and Validate
  workspaces for level metadata, prop transforms, gameplay fields, guided
  system editing, raw JSON escape hatches, and issue-to-prop navigation
- use Prefabs to capture the current selection around its bottom-center pivot,
  then place the validated group from Camera, Top, Front, or Side as one Undo
  operation
- use `F` to focus selected, `Ctrl+D` to duplicate, and `Ctrl+C`/`Ctrl+V` to
  copy/paste selected props with the default bindings
- `Validate` checks the current unsaved level through the Rust validator
- `Save` writes the real level file only after validation passes
- undo/redo coalesces a drag or focused field edit into one history step and
  reports Saved again when history returns exactly to the on-disk snapshot
- unsaved edits are stored as a browser-local draft; reopening the level offers
  recovery and warns when the on-disk level changed since the draft began

Runtime quick-adjust controls:

- `Tab` toggle in-game level editor
- `G` cycle editor mode: geometry, items, enemies, entities
- left/right arrows cycle the current placement template
- up/down arrows select existing props
- mouse wheel adjusts placement distance
- `Enter` place the current template at the snapped cursor
- `Delete` remove the selected or nearest prop
- `V` validate the current level in-editor
- `P` save the current `levels/<level>.json` after validation passes
- `R` hot reload the current level from disk

Editor-supported authoring data now includes stable prop IDs, imported asset
metadata, loot tables, authored paths, level events, and dialogue blocks. The
runtime can execute on-enter/proximity events, grant resources, spawn weighted
loot rolls, queue level transitions, log dialogue, set/save level-local flags,
restore once-event state on continue, and move path-bound enemies/props.
The editor HUD shows saved/unsaved state, validation state, cursor coordinates,
selected prop count, placement distance, and the core edit keys while active.

The editor expansion checklist lives in `docs/editor/LEVEL_EDITOR_BACKLOG.md`.

The in-game HUD now mirrors these core verbs with a compact guide strip, thicker
high-contrast block text, shot-result feedback, and the event feed.

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

The current build is still a foundation prototype, not the First Ascent
Prototype.

Implemented now:

- movement, sprint, dash, pause, readable guided HUD, standalone level editor,
  in-game quick adjuster, audio feedback, level loading, config loading,
  hurtbox damage, death/respawn, prototype prop shooting with solid-world
  obstruction, baseline data-driven enemy chase/attack, prototype resource
  pickup/Anchor banking, authored relic pickups, deterministic enemy relic
  rewards, owned-relic cycling, autosave/resume, basic cycle modifiers, and
  full project content validation

Not implemented yet:

- advanced enemy AI, role-specific enemy behavior, production enemy models,
  generated relic rolls/affixes, item comparison UI, full Anchor/Sanctuary UI,
  full run-contract Cycle Director, and procedural route generation


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
