# Cenotaph Roadmap

## Purpose

This document defines the staged development plan.

The roadmap should prevent the forever project from becoming shapeless.

Cenotaph can grow forever, but it must always have a playable current milestone.

---

# Roadmap Rule

Do not move to the next stage until the current stage answers its success question.

A stage is complete when the game proves the design question for that stage, not when every possible feature is finished.

# Stage 0: Foundation Prototype

Status: In progress as the current foundation build.

Current notes:

- Movement, camera, jump, sprint, dash, pause, readable guided HUD, hurtbox
  damage, death, respawn, prototype prop shooting with
  solid-world obstruction, baseline enemy chase/attack, authored relic
  pickups/rewards, owned-relic cycling, autosave/resume, basic cycle modifiers,
  and full project content validation exist.
- Starter enemy definitions use validation-enforced single-primitive
  placeholders; no generated enemy design is treated as canonical art.
- Minimal resource pickup, explicit Anchor rites/respawn, relic pickup, and hazard
  examples exist in `foundation_test` and `movement_test`.
- Advanced enemy AI, role-specific enemy behavior, and production enemy models
  do not exist yet.

## Goal

Prove basic feel.

## Required

- Player movement
- Camera
- Jumping
- Falling
- One weapon
- One authored baseline enemy role
- Health
- Damage
- Death
- One test arena
- In-game level editing tools for maintaining the test arena

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
- [ ] Basic combat feels understandable, including hit, kill, miss, and
  blocked-shot feedback.

## Future Notes

Use `levels/foundation_test.json` as the permanent Stage 0 smoke-test arena.
Do not promote Stage 0 to complete until the authored enemy can be manually
smoke-tested as readable model/collider/tuning inside the arena.

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
- One authored baseline enemy role
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
- [ ] Enemy can drop a prototype authored relic.
- [ ] Player can intentionally equip/cycle owned relics.
- [ ] Player can reach an Anchor.
- [ ] Player can bank a resource.
- [ ] Player loses unsecured resource on death.
- [ ] A run modifier changes the second attempt.
- [ ] The same small climb is worth replaying at least a few times.

## Future Notes

`ashwalk_01` is now a compact single-route pilgrimage slice with a safe start,
one authored hazard, paced Ash, one field relic, the named Ash-Warden encounter,
the guaranteed `Debt of the Last Keeper` drop, and an explicit summit Anchor
rite. The Warden's death and the first Anchor claim trigger authored mountain
reactions through reusable level-event data; overlapping answers queue in order.
Named encounter/reward HUD and world-state resume reconstruction preserve the
identity of the Warden and its relic across the full loop. The slice is
protected by content, runtime-preparation, save reconstruction,
reaction-envelope, and HUD-layout tests but still needs a full manual playtest.

A meaningful two-route decision remains an explicit Stage 1 exit criterion.
Next tune this route's combat pacing and rite cost in play, then add the route
split and one behavior-changing relic or perk decision. Do not add another
stratum before this climb is worth replaying.

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

- Ashbound
- Burdened
- Censer
- Chainrunner
- Silencer
- Paranoiac
- Harpy
- Bellworn
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
- Four Ash-Walk enemy roles: Ashbound, Burdened, Censer, Harpy
- One boss or named elite
- World event table
- Authored campaign completion boundary; death and NG+ rules remain UNDEFINED
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
- [ ] The authored Ash-Walk slice has a defined completion boundary.
- [ ] Save/load works.
- [ ] Lore is present but not over-explained.
- [ ] The slice feels replayable.

## Future Notes

Add notes here during implementation.

---

# Stage 6: Campaign Replayability and New Game+

## Goal

Make campaign replay, New Game+, and optional endgame meaningfully different
without replacing the authored world with procedural maps.

## Required

- Campaign completion state
- Explicit New Game+ design
- Optional endgame contract
- Variable enemy tables
- Variable relic pools
- Variable hazards
- Rare events
- Persistent discoveries and world changes where explicitly designed

## Success Question

```text
Can players replay the authored campaign with meaningful build, loot, threat,
and optional endgame variation?
```

## Exit Criteria

- [ ] Campaign replay and NG+ rules are explicitly defined.
- [ ] Run contracts, if retained, serve replay rather than replacing campaign progression.
- [ ] Enemy tables can shift.
- [ ] Relic pools can shift.
- [ ] Hazards can shift.
- [ ] Events can shift.
- [ ] Explicitly approved progression persists according to documented NG+ rules.
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
- Campaign replay mutations, if approved
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
- Campaign replayability and optional endgame contract
- Save System

## Potential Expansion: Lore Archive

Status: Future

Depends on:

- Narrative Discovery
- Save System
- UI System
