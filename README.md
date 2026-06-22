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

## Core Player Loop

```text
Enter the mountain
Choose a route
Fight through hostile spaces
Recover relics
Adapt the build
Survive pressure
Reach an Anchor or Sanctuary
Bank progress
Push higher
Begin another ascent
```

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

This README is the master overview.

Detailed documents live in the following files:

- `SYSTEMS.md` — the mechanical and technical systems map.
- `CONTENT_GUIDE.md` — rules and templates for adding new content.
- `LORE.md` — mythology, tone, dream logic, and narrative rules.
- `ROADMAP.md` — staged development plan and milestone definitions.
- `TECHNICAL_NOTES.md` — Rust architecture direction, data registry, validation, save structure, and developer tools.

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

See `SYSTEMS.md` for the full breakdown.

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

Each Stratum should be documented uniformly using the template in `CONTENT_GUIDE.md`.

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

## Future Additions

- [ ] Screenshots
- [ ] Build instructions
- [ ] Controls
- [ ] Current playable version
- [ ] Known issues


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
