# Cenotaph Roadmap

## Purpose

This document defines the staged development plan.

The roadmap should prevent the forever project from becoming shapeless.

Cenotaph can grow forever, but it must always have a playable current milestone.

---

# Roadmap Rule

Do not move to the next stage until the current stage answers its success question.

A stage is complete when the game proves the design question for that stage, not when every possible feature is finished.

---

# Stage 0: Foundation Prototype

## Goal

Prove basic feel.

## Required

- Player movement
- Camera
- Jumping
- Falling
- One weapon
- One enemy
- Health
- Damage
- Death
- One test arena

## Success Question

```text
Does it feel good to move, shoot, get hit, and defeat an enemy?
```

## Exit Criteria

- [ ] Player can move reliably.
- [ ] Camera feels usable.
- [ ] Player can fire a weapon.
- [ ] Enemy can take damage.
- [ ] Enemy can damage player.
- [ ] Player can die.
- [ ] Basic combat feels understandable.

## Future Notes

Add notes here during implementation.

---

# Stage 1: First Ascent Prototype

## Goal

Prove the core loop.

## Required

- One small vertical test Stratum
- One Anchor
- One safe room
- Two routes
- One weapon family
- Basic relic generation
- Basic loot drop
- Equip / replace relic
- One enemy
- One elite modifier
- One hazard
- One resource
- Resource loss on death
- Resource banking
- One upgrade or perk choice
- One run modifier

## Success Question

```text
Is it fun to climb, survive, find a relic, adapt, and try again?
```

## Exit Criteria

- [ ] Player can start at safe point.
- [ ] Player can choose between at least two routes.
- [ ] Player can fight through danger.
- [ ] Enemy can drop a generated relic.
- [ ] Player can equip the relic.
- [ ] Player can reach an Anchor.
- [ ] Player can bank a resource.
- [ ] Player loses unsecured resource on death.
- [ ] A run modifier changes the second attempt.
- [ ] The same small climb is worth replaying at least a few times.

## Future Notes

Add notes here during implementation.

---

# Stage 2: Loot Identity

## Goal

Make loot feel like the heart of the game.

## Required

- Three weapon families
- Rarity tiers
- Prefixes
- Suffixes
- Elements
- Trait packages
- Item names
- Item comparison
- Loot feedback
- Basic loot tables

## Success Question

```text
Does finding a relic create excitement and build curiosity?
```

## Exit Criteria

- [ ] Player can identify rarity immediately.
- [ ] Relics have readable differences.
- [ ] At least three weapon families feel distinct.
- [ ] Prefixes and suffixes change behavior or build value.
- [ ] Item comparison is clear enough for decision-making.
- [ ] At least one relic makes the player want to change approach.

## Future Notes

Add notes here during implementation.

---

# Stage 3: Buildcraft

## Goal

Make player choices meaningfully affect survival.

## Required

- Core stats
- Perks
- Status effects
- Relic/perk synergies
- Build-defining bonuses
- Enemy counters
- Basic respec or experimentation support

## Success Question

```text
Can two players survive the same route with different builds?
```

## Exit Criteria

- [ ] Perks affect real gameplay decisions.
- [ ] Relics interact with perks.
- [ ] At least three viable build directions exist.
- [ ] Status effects matter.
- [ ] Enemies create build pressure.
- [ ] Player can understand why a build works or fails.

## Future Notes

Add notes here during implementation.

---

# Stage 4: Threat Depth

## Goal

Make the mountain mechanically oppressive.

## Required

- Burdened
- Silencer
- Paranoiac
- Harpy
- Enemy modifiers
- Elite variants
- Suppression
- Encounter tables
- Multiple hazards

## Success Question

```text
Do enemies attack the player’s options, not only their health?
```

## Exit Criteria

- [ ] Each core enemy has a distinct combat role.
- [ ] At least one enemy suppresses player agency.
- [ ] At least one enemy controls vertical space.
- [ ] At least one enemy escalates encounters.
- [ ] Elite modifiers change behavior or counterplay.
- [ ] Hazards interact with combat or routing.

## Future Notes

Add notes here during implementation.

---

# Stage 5: Ash-Walk Pilgrimage

## Goal

Create the first true vertical slice.

## Required

- Complete Ash-Walk Stratum
- Sanctuary prototype
- Multiple routes
- Three weapon families
- Three enemy types
- One boss or named elite
- World event table
- Cycle reset
- 10–20 lore fragments
- 1–3 named relics
- Save persistence

## Success Question

```text
Does this feel like a small but real version of Cenotaph?
```

## Exit Criteria

- [ ] Ash-Walk has a clear visual and gameplay identity.
- [ ] Sanctuary provides relief and preparation.
- [ ] Multiple route choices matter.
- [ ] Relics create build experimentation.
- [ ] Enemies create varied pressure.
- [ ] One boss or named elite anchors the slice.
- [ ] Cycle reset works.
- [ ] Save/load works.
- [ ] Lore is present but not over-explained.
- [ ] The slice feels replayable.

## Future Notes

Add notes here during implementation.

---

# Stage 6: Replayability Spine

## Goal

Make repeated ascents meaningfully different.

## Required

- Run Contract system
- Cycle Director
- Cycle mutations
- Variable enemy tables
- Variable relic pools
- Variable hazards
- Rare events
- Persistent memory changes

## Success Question

```text
Can players describe one ascent differently from another?
```

## Exit Criteria

- [ ] Run contracts generate distinct ascent identities.
- [ ] Enemy tables can shift.
- [ ] Relic pools can shift.
- [ ] Hazards can shift.
- [ ] Events can shift.
- [ ] Some changes persist between cycles.
- [ ] Replays feel meaningfully different without relying on huge new maps.

## Future Notes

Add notes here during implementation.

---

# Stage 7: Forever Expansion

## Goal

Grow the mountain safely over time.

## Add Through Content Packs

- New strata
- New relics
- New enemies
- New modifiers
- New bosses
- New events
- New hazards
- New lore fragments
- New cycle mutations
- New named relics

## Success Question

```text
Can new content be added without destabilizing the core?
```

## Exit Criteria

This stage never truly ends.

A content addition is successful when:

- [ ] It supports at least one core pillar.
- [ ] It passes validation.
- [ ] It works inside an ascent.
- [ ] It does not require rewriting the core.
- [ ] It can interact with existing systems.

---

# Current North Star

```text
First Ascent Prototype
```

Do not build the entire game before proving this.

---

# Systems To Avoid Early

Do not build these first:

- Multiplayer
- Online trading
- Huge open world
- Seven complete strata
- Full cinematic story
- Complex crafting
- Escort missions
- Large stealth systems
- Advanced NPC schedules
- Full mod support
- Massive procedural terrain
- Hundreds of weapons
- Console support
- Leaderboards

These may become valid later.

They are not foundation systems.

---

# Roadmap Backlog

Use this section for future stages or major expansions.

## Potential Expansion: Ward of Irons Pack

Status: Future

Depends on:

- Ash-Walk Pilgrimage
- Replayability Spine
- Hazard system stability

## Potential Expansion: Named Relic Season

Status: Future

Depends on:

- Relic System
- Build System
- Content Validation

## Potential Expansion: Boss Memory Variants

Status: Future

Depends on:

- Boss System
- Cycle Director
- Save System

## Potential Expansion: Lore Archive

Status: Future

Depends on:

- Narrative Discovery
- Save System
- UI System
