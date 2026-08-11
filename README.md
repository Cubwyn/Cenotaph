# Cenotaph: The Great Omission

## Master Project README

---

## One-Sentence Vision

**Cenotaph is a surreal authored looter-shooter RPG campaign through a hostile metaphysical mountain, with long-term loot, buildcraft, optional New Game+, and optional endgame replay.**

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

- Alone
- Small
- Forgotten
- Curious
- Pressured
- Determined
- Increasingly capable, but never fully safe, celebrated, or larger than the mountain

The game should feel like climbing through a dream that has rules, but refuses to explain them.

---

## Narrative Spine

This section records developer-facing story truth. The game should reveal it
slowly through play, relic memories, environments, NPC fragments, bosses, and
Cycle changes rather than presenting it as an opening explanation.

### The Cenotaph

The Cenotaph is the mountain.

It is also the remains of a forgotten kingdom and civilization: its roads,
settlements, cathedrals, machinery, courts, and wounds compressed into an
impossible ascent.

The mountain is a cenotaph for a kingdom that was never allowed to finish
dying. Its civilization persists in surreal agony because its final reckoning
was abandoned. The mountain rearranges and repeats because the kingdom's ending
is still unresolved.

The Great Omission is the absence at the center of that failed ending. Its full
nature should remain mysterious for much of the game, but it is connected to
the missing king, his sin, and the judgment he refused to face.

### The Pilgrim-King

The player begins as a nameless pilgrim without memories.

The hidden truth is that the pilgrim is the former king of the mountain. He
committed a sin so shameful that he fled rather than remain and answer for it.
By abandoning his final obligation, he doomed the kingdom to an existence that
could neither continue nor end.

The exact sin is an author-level mystery still to be defined. It must explain:

- why the king fled;
- why his absence prevented the kingdom's death;
- what the Great Omission removed or left unfinished;
- why the mountain returns him as a pilgrim;
- what payment can finally conclude the kingdom's suffering.

The story is not about reclaiming the throne. It is about becoming answerable
to it.

### The Emotional Contract

The player should always feel alone, small, and forgotten, even when powerful,
surrounded, or temporarily safe.

- **Alone:** no companion shares the full ascent. NPC encounters are rare,
  limited, unreliable, or unable to follow.
- **Small:** architecture, height, danger, and history remain greater than the
  player. Power improves survival without turning the game into triumphal power
  fantasy.
- **Forgotten:** names are missing, records are damaged, NPCs misremember, and
  Anchors may preserve passage without preserving the person who made it.

These feelings are consequences of the king's failure. He abandoned those who
depended on him, placed himself above his subjects, and removed himself from
their final history. Each Cycle forces him to experience the kingdom from the
position of its smallest forgotten pilgrim.

The kingdom forgot its king. It did not forget what he left unfinished.

### Cycles and Atonement

Each ordinary Cycle returns the former king without his royal name, memories,
or authority. He climbs through the consequences of his reign, reaches the
upper mountain, and fails to complete the necessary reckoning. The kingdom
therefore repeats.

An ordinary Cycle may end because the pilgrim repeats some form of the original
failure: fleeing, choosing power over responsibility, refusing his identity,
reclaiming authority without accepting guilt, destroying a necessary witness,
or arriving without sufficient understanding.

The final ending requires more than reaching the summit or collecting a fixed
set of objects. Through persistent choices across Cycles, the player must prove
that he has stopped behaving like the king who fled. He must recover enough
truth to understand the crime, witness its consequences, complete abandoned
obligations, accept his identity without reclaiming royal authority, relinquish
something of real value, and remain when escape would be easier.

At the final rite, the player does not conquer the mountain. He accepts judgment
and pays for his sin. His payment ends his recurrence and finally allows the
kingdom, the mountain, and the Cenotaph to die.

The true victory is release, not possession.

### Mystery and Gameplay Clarity

Cenotaph may be surreal in meaning, history, imagery, space, and memory. It
must not become confusing to play.

The player may be uncertain about what something means, but never about what
they must do or how a mechanic behaves.

- Objectives use direct action language.
- Hazards, interactions, costs, routes, and rewards have readable rules.
- Establish a rule before distorting it.
- Introduce one major strange idea at a time.
- Give NPCs understandable desires even when their memories are wrong.
- Deliver major revelations through concrete evidence as well as symbolism.
- Every contradiction has an intended narrative reason.
- Reality distortion never excuses unreadable combat, traversal, UI, or state.

Ash-Walk begins with physical, understandable danger. Surreal complexity grows
with altitude, reaches its greatest concentration in Mirror-Crust and the
Breach, and always remains mechanically legible.

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
12. The player may be uncertain about meaning, but never about the immediate objective or mechanical rule.
13. Surrealism must be patterned, readable, and consequential rather than random.
14. The player grows more capable without ceasing to feel alone, small, and forgotten.
15. The final narrative reward is accountability and release, not restored kingship.

---

## Documentation Map

This README is the master overview. The categorized documentation index lives
at [`docs/README.md`](docs/README.md):

- `docs/design/IDENTITY_CONTRACT.md` is the top-level identity guardrail for
  future design, content, and coding-agent work.
- `docs/design/ASH_WALK_PILGRIMAGE.md` is the current playable milestone and
  acceptance gate before adding another Stratum.
- `docs/design/CENOTAPH_STRATA.md` defines the themes, visual identities,
  gameplay roles, threats, routes, hazards, relic biases, Cycle mutations, and
  development purposes of all seven Strata.
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
cargo content            # Show the content map and safe change loop
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
5. **Campaign Replayability** — authored campaign progression, optional New Game+, endgame variation, and persistent memory where explicitly defined.

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

| Stratum | Theme | Narrative and development role |
| --- | --- | --- |
| **1. Ash-Walk** | Collapse, ash, and first survival | Introduces the scale of the mountain, basic ascent, relic hunting, route choice, and the first signs that the mountain responds to the pilgrim. |
| **2. Ward of Irons** | Industrial confinement and machinery without purpose | Shows systems still obeying an absent authority and adds confined environmental pressure. |
| **3. Hanging Slums** | Precarious life above oblivion | Reveals how ordinary people attempted to live inside the climb and suffered after their king abandoned them. |
| **4. Sanctuary** | Temporary safety, melancholy, and preparation | Provides relief and long-term preparation while preserving incompatible stories about the missing king. Safety does not become belonging. |
| **5. Gallery of Wind** | Exposure, height, and insignificance | Expands verticality and confronts the player with the kingdom's impossible scale without becoming a pure movement shooter. |
| **6. Mirror-Crust** | Reality distortion, reflection, and uncertainty | Turns accumulated clues toward recognition of the pilgrim's identity while keeping illusions and routes mechanically readable. |
| **7. The Breach** | Final ascent and reality failing to hold its shape | Brings the consequences of every Stratum together for endgame combat, Cycle completion, and the possibility of final atonement. |

Each Stratum should be documented uniformly using the template in `docs/design/CONTENT_GUIDE.md`.

The narrative progression is:

```text
Physical ruin
-> purposeless authority
-> abandoned subjects
-> safety without belonging
-> overwhelming scale
-> recognition
-> judgment
```

Ash-Walk is only the first Stratum. It teaches the mountain's basic grammar and
must not consume the visual, mechanical, enemy, relic, or narrative identities
reserved for higher Strata.

---

## Replayability Principle

Replayability is a first-class design pillar, but the authored campaign remains
the primary structure. New Game+ and optional endgame replay must add long-term
loot and build depth without replacing the mountain with an endless procedural
run loop.

Cycles may remain as lore or explicitly designed optional systems. They are not
the primary progression structure.

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
  rewards, named encounter/reward HUD, world-state autosave/resume, and prototype Cycle modifiers pending the
  campaign/NG+ design decisions,
  full project content validation; Ashwalk now composes those systems into a
  compact readable climb with one hazard, paced resources, a relic choice, an
  authored named Ash-Warden and relic drop, a summit rite, reusable mountain
  reactions, and a return gate

Not implemented yet:

- advanced enemy AI, role-specific enemy behavior, production enemy models,
  generated relic rolls/affixes, item comparison UI, data-authored Anchor rite
  variants and full Sanctuary UI,
  a data-driven elite modifier, upgrade/perk choice, campaign completion and
  New Game+ rules, the replacement replayability system, and procedural route
  generation

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

The pilgrim climbs without knowing that he was once the king who fled.

The mountain forgets.

The player remembers.

The relics remember incorrectly.

The kingdom remembers only the wound left by its absent sovereign.

Every ordinary Cycle repeats the failure.

The final Cycle ends when the king remains, answers for his sin, and allows the
Cenotaph to die.

And every ascent becomes another version of the same impossible dream.

---
A solo project by Cubwyn
