# Cenotaph: The Great Omission

Cenotaph is a solo-developed **looter-shooter RPG** set on a massive, ruined mountain.

The mountain is made from the remains of a forgotten kingdom: buried roads, broken cathedrals, abandoned machinery, hanging settlements, and places where reality has started to behave strangely.

You climb it because there is nowhere else to go.

The game is heavily inspired by games like **Borderlands** and **Dark Souls**, but Cenotaph is meant to have its own identity. Borderlands is mainly an inspiration for loot and build variety, while Dark Souls is an inspiration for atmosphere, world design, danger, and mystery.

---

## The Game

The basic idea is pretty simple:

```text
Explore
  ↓
Fight
  ↓
Find relics
  ↓
Improve your build
  ↓
Choose where to go
  ↓
Keep climbing
```

The individual levels are not meant to be straight paths from A to B.

The overall mountain has a set progression through its major Strata, but the levels within them are built around **branching routes, shortcuts, optional areas, hidden loot, encounters and different ways through the same space**.

The goal is to make each level feel like an actual place rather than a series of combat arenas.

---

## Loot

Loot is one of the main reasons to keep playing.

Weapons and relics are meant to be more than simple stat upgrades. Builds should come from finding things that work well together, with relics providing unusual effects, bonuses and tradeoffs.

The loot system is still being developed, but the general direction is:

* Named weapons and relics
* Rarity
* Affixes
* Build synergies
* Strange effects
* Risk/reward mechanics
* Relics connected to the world and its history

A good drop should make you think about what you could build around it.

---

## The Mountain

The setting is just as important as the gameplay.

Cenotaph takes place almost entirely on one enormous mountain. Instead of travelling across a normal open world, the player is gradually pushed higher through different parts of the same ruined civilization.

The further up you go, the stranger things become.

Early areas are relatively understandable: ruins, ash, roads, machinery and settlements.

Later areas become increasingly surreal, with things like distorted spaces, impossible architecture, reflections, broken gravity and memories that don't quite agree with reality.

The world should feel like it has rules even when the player doesn't understand those rules yet.

---

## The Great Omission

Something happened to the world.

Something was removed, forgotten, or never allowed to exist properly.

That absence is known as **The Great Omission**.

The player is not given the whole story at the beginning. Most of it is discovered through the environment, relics, NPCs, bosses and things that don't quite make sense until later.

The intention is for the player to slowly piece together what happened rather than having the game explain its mythology directly.

---

## Strata

The mountain is divided into seven major Strata.

### 1. Ash-Walk

**Collapse, ash and first survival.**

The first major area of the mountain. Buried roads, collapsed settlements, broken shrines and ash fields.

This is where the player learns the basic systems and starts seeing that the mountain is much larger than expected.

### 2. Ward of Irons

**Industrial confinement and machinery without purpose.**

Rusted machinery, pipes, steam, iron corridors and enormous vertical shafts.

Combat becomes more confined and environmental hazards become more important.

### 3. Hanging Slums

**Life above oblivion.**

A settlement built into the mountain.

Hanging houses, rope bridges, chains, broken lifts and large drops make route choice and exploration more important.

### 4. Sanctuary

**A place to stop climbing for a while.**

A suspended cathedral and one of the few places where the mountain isn't actively trying to kill you.

This acts as a hub for recovery, preparation, NPCs and progression.

### 5. Gallery of Wind

**Height and exposure.**

Large open spaces, floating ruins and powerful wind currents.

The focus shifts towards long sightlines, aerial enemies and more dangerous vertical routes.

### 6. Mirror-Crust

**Reality starting to break down.**

Reflections, duplicated architecture, inverted spaces and routes that don't behave quite how they should.

This is where the more surreal side of Cenotaph becomes much more prominent.

### 7. The Breach

**The end of the mountain.**

The place where the mountain can no longer hold itself together.

Broken terrain, impossible geometry, voids and the final encounters.

---

## Tone

Cenotaph is supposed to be dark, strange and oppressive without becoming generic grimdark.

The main influences are:

* Dark fantasy
* Surrealism
* Progressive rock
* Soulslike environmental storytelling
* Looter RPGs
* Decaying architecture
* Mythology

Visually, the game leans towards things like ash, black stone, iron, bone, tarnished gold, old machinery and enormous structures disappearing into fog.

Simple or low-poly assets are intentional during development, but the goal isn't to make the game look cheap. Composition, lighting, scale, fog and silhouettes are more important than having extremely detailed models.

---

## Development

Cenotaph is being made by one person, so the project is deliberately built around reusable systems and assets.

The engine and game are written primarily in **Rust**.

Current technology includes:

* Rust 2021
* `wgpu`
* `winit`
* Rapier3D
* `rodio`
* JSON level data
* TOML configuration
* Data-driven enemies and relics

The project is still in the prototype stage.

There is already a working foundation with movement, combat, enemies, relics, Anchors, level loading, events, saving and the first Ash-Walk test level.

The important part right now is not making a huge amount of content.

It is getting the **first proper ascent** working and making sure it actually feels like Cenotaph.

---

## Project Structure

```text
assets/          Runtime assets
config/          Controls and gameplay settings
data/            Enemy and relic definitions
docs/            Design, lore and development documentation
levels/          Level data
prefabs/         Reusable level components
scripts/         Project and validation scripts
source_assets/   Source files and references
src/             Rust source code
textures/        Runtime textures
```

The more detailed design rules live in `docs/`.

The [Identity Contract](docs/design/IDENTITY_CONTRACT.md) is the main guardrail for the project. It exists to prevent new systems or content from slowly turning Cenotaph into something else.

---

## Running

The project currently uses Cargo.

```powershell
cargo run
```

Run the Ash-Walk test level:

```powershell
cargo run -- play ashwalk_01
```

Run the foundation test:

```powershell
cargo run -- play foundation_test
```

Resume the latest save:

```powershell
cargo run -- continue
```

Validate project content:

```powershell
cargo run -- validate
```

Run project diagnostics:

```powershell
cargo doctor
```

List playable levels:

```powershell
cargo levels
```

---

## What I'm Working On

The current priority is **Ash-Walk** and the first complete ascent.

That means getting the following parts working together properly:

* Branching level layout
* Combat
* Enemies
* Relic drops
* Build decisions
* Hazards
* Anchors
* Exploration
* Rewards
* A reason to keep climbing

Once that works, the same foundation can be used to build the rest of the mountain.

The project is intentionally being developed from the systems outward rather than trying to build all seven Strata at once.

---

## Status

**Prototype / Active Development**

Cenotaph is nowhere near finished.

A lot of the systems and content described in the design documents are still planned rather than implemented. The current build is mainly there to prove that the core idea works before the project gets larger.

---

## Links

* `docs/design/` — Game design
* `docs/lore/` — Lore and story
* `docs/development/` — Technical documentation
* `docs/art/` — Art direction and asset guidelines

---

**Cenotaph: The Great Omission**

*A solo project by Cubwyn.*
