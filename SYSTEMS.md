# Cenotaph Systems Document

## Purpose

This document defines the main gameplay and technical systems of Cenotaph.

It should stay practical.

Lore and dream logic belong in `LORE.md`.

Content templates belong in `CONTENT_GUIDE.md`.

Implementation notes belong in `TECHNICAL_NOTES.md`.

---

# System Hierarchy

Cenotaph is built around five master systems:

1. Ascent System
2. Relic System
3. Build System
4. Threat System
5. Cycle Director

Supporting systems:

6. Data Registry
7. Content Validation
8. Save System
9. Presentation System
10. Developer Tools

---

# 1. Ascent System

## Purpose

Controls the climb itself.

The Ascent System answers:

```text
What does the player do, where do they go, and what happens when they fail or survive?
```

## Includes

- Strata
- Routes
- Anchors
- Sanctuary
- Death
- Respawn
- Resource banking
- Route risk
- Survival pressure
- Climb progression

## Core Loop

```text
Start at Anchor or Sanctuary
Choose route
Enter danger
Fight / explore / recover relics
Decide whether to continue or return
Reach next Anchor
Bank progress
Push upward
```

## Key Objects

### Anchor

A checkpoint and banking point.

Functions:

- Respawn point
- Partial recovery
- Resource banking
- Route checkpoint
- Possible fast travel node

### Sanctuary

A safe hub.

Functions:

- Inventory management
- Perk allocation
- Storage
- Vendors
- NPC dialogue
- Objective board
- Cycle transition point

### Routes

Route types:

- High Route — exposed, dangerous, vertical, better rewards.
- Mid Route — balanced, readable, standard progression.
- Deep Route — hidden, oppressive, resource-heavy, secret-rich.

## MVP Requirement

- One small vertical test area
- One Anchor
- One safe room
- Two route choices
- Death and respawn
- Resource loss or unsecured resource mechanic
- One reason to continue climbing

## Expansion Slots

- [ ] Multiple Anchors
- [ ] Full Sanctuary
- [ ] Stratum route networks
- [ ] Shortcuts
- [ ] Hidden paths
- [ ] Route-specific events
- [ ] Route-specific relic tables
- [ ] Route-specific enemy pressure

---

# 2. Relic System

## Purpose

Controls loot, weapons, rarity, drops, inventory, and item identity.

The Relic System answers:

```text
What did the player find, and why is it exciting?
```

## Includes

- Weapon generation
- Loot drops
- Rarity
- Affixes
- Named relics
- Item identity
- Inventory
- Comparison
- Drop tables
- Relic presentation

## Relic Structure

```text
Weapon Family
+
Base Archetype
+
Rarity
+
Prefix
+
Suffix
+
Element
+
Trait Package
+
Stat Rolls
+
Name
+
Flavor Text
```

## Weapon Families

### Sovereign

Heavy, precise, authoritative weapons.

Design role:

- High damage
- Strong recoil
- High impact
- Precision or heavy firepower

### Moonchild

Dreamlike weapons with impossible behavior.

Design role:

- Tracking
- Curving projectiles
- Delayed effects
- Strange utility

### Schizoid

Chaotic and unstable weapons.

Design role:

- Volatility
- Randomized behavior
- Explosive output
- High risk and high reward

### Named Relics

Rare unique weapons with special mechanics.

Named relics should be memorable chase items.

## Suggested Rarity Tiers

```text
Common
Rare
Epic
Legendary
Mythic
Transcendent
```

## MVP Requirement

- One weapon family
- Basic random stat rolls
- Rarity tiers
- Loot drops from enemies
- Equip / replace relic
- Simple inventory display

## Expansion Slots

- [ ] Three weapon families
- [ ] Full affix system
- [ ] Named relics
- [ ] Relic curses
- [ ] Cycle-exclusive relics
- [ ] Movement-altering relics
- [ ] Relics with lore memories
- [ ] Relic collection archive
- [ ] Loot filters
- [ ] Relic rerolling or refinement

---

# 3. Build System

## Purpose

Controls adaptation, progression, stats, perks, status effects, and synergies.

The Build System answers:

```text
How does the player adapt?
```

## Includes

- Player stats
- Weapon stats
- Enemy stats
- Perks
- Synergies
- Status effects
- Damage formulas
- Movement modifiers
- Survival modifiers
- Risk/reward mechanics

## Perk Paths

### Ascension

Focus:

- Height
- Positioning
- Recoil mastery
- Movement efficiency
- Vertical combat

### Memory

Focus:

- Sustain
- Recovery
- Loot
- Economy
- Resource banking
- Discovery

### Omission

Focus:

- Forbidden power
- Corruption
- Risk/reward
- Suppression conversion
- Reality-breaking effects

## Status Effects

Initial candidates:

- Burn
- Frost
- Shock
- Void
- Gravity
- Time
- Silence
- Corruption
- Fracture

## Synergy Examples

```text
High recoil Sovereign weapon
+
Ascension recoil perk
=
Vertical repositioning build
```

```text
Life-steal relic
+
Memory sustain perk
=
Long-route survival build
```

```text
Silence field relic
+
Omission suppression perk
=
Power spike when abilities are disabled
```

## MVP Requirement

- Basic player stats
- Basic weapon stats
- One perk or upgrade choice
- One meaningful relic/perk synergy
- One status effect

## Expansion Slots

- [ ] Three perk paths
- [ ] Keystone perks
- [ ] Full status system
- [ ] Build-defining relics
- [ ] Perk respec
- [ ] Build loadouts
- [ ] Hidden synergies
- [ ] Cycle-specific build pressure

---

# 4. Threat System

## Purpose

Controls enemies, encounters, hazards, bosses, and suppression.

The Threat System answers:

```text
What is trying to stop the player?
```

## Includes

- Enemies
- Enemy AI
- Enemy modifiers
- Elites
- Bosses
- Hazards
- Encounters
- Suppression
- Enemy loot tables

## Core Enemy Roles

The full gameplay-first roster lives in `ENEMY_GAMEPLAY_ROSTER.txt`.

Core combat jobs:

- Ashbound: grunt and baseline melee pressure.
- Burdened: tank and route blocker.
- Censer: glass cannon and priority target.
- Chainrunner: high-speed flanker.
- Harpy: aerial and vertical threat.
- Bellworn: ranged harasser.
- Silencer: suppression enemy.
- Paranoiac: encounter escalator.
- Anchor Parasite: swarmer.
- Root-Machine Hybrid: arena controller.
- Mirror of the Player: late-game duelist.
- Bell-Headed: elite modifier frame.

The early Ash-Walk slice should focus on Ashbound, Burdened, Censer, Harpy, and
one elite Burdened-style modifier before adding suppression or late-game mirror
enemies.

## Enemy Modifier Structure

```text
Base Enemy
+
Modifier Package
+
Cycle Scaling
=
Enemy Variant
```

Example variants:

- Burning Burdened
- Mirror Harpy
- Silent Paranoiac
- Void-Touched Silencer
- Armored Burdened
- Frenzied Harpy

## Hazard Examples

- Ash storms
- Chain tremors
- Crushing machinery
- Wind currents
- Silence fields
- Gravity distortions
- Mirror illusions
- Void fractures
- Collapsing platforms

## Suppression Effects

Suppression may affect:

- Abilities
- Perks
- Recoil movement
- Healing
- Visibility
- Audio
- Navigation
- UI certainty

## MVP Requirement

- One enemy from the gameplay roster
- One elite modifier
- One environmental hazard
- One suppression or pressure mechanic
- One encounter zone

## Expansion Slots

- [ ] Full enemy roster
- [ ] Multiple modifiers
- [ ] Elite variants
- [ ] Bosses
- [ ] Stratum-specific hazards
- [ ] Encounter escalation
- [ ] Enemy route control
- [ ] Boss suppression phases

---

# 5. Cycle Director

## Purpose

Controls replayability, run generation, mutations, and long-term variation.

The Cycle Director answers:

```text
Why is this ascent different from the last one?
```

## Includes

- Cycle reset
- Run contracts
- Enemy table changes
- Relic pool changes
- Event selection
- Stratum mutations
- Difficulty scaling
- Persistent memory
- Rare conditions
- Long-term unlocks

## Run Contract

Every ascent should have an internal identity.

Example:

```text
Cycle: 4
Primary Stratum: Ash-Walk
Route Bias: High Route
Threat Bias: Harpies + Silencers
World Event: Ash Storm
Relic Bias: Sovereign
Mutation: Healing reduced near elites
Secret: Mirror cache active
```

## Replayability Formula

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

Each ascent should change at least three of these.

## Good Cycle Changes

- Silencers appear earlier.
- Ash storms can hide elites.
- Mirror caches appear in lower routes.
- Named relics can mutate.
- Sanctuary dialogue changes.
- A defeated boss leaves a permanent scar.

## Weak Cycle Changes

- Enemies have only more health.
- Drop rates increase with no new decision-making.
- Hazards deal more damage but behave identically.

## MVP Requirement

- One reset loop
- One run modifier
- One enemy modifier
- One loot table variation
- Basic retained progression

## Expansion Slots

- [ ] Cycle milestones
- [ ] Rare event chains
- [ ] Cycle-exclusive relics
- [ ] Stratum mutations
- [ ] Boss memory variants
- [ ] Persistent world scars
- [ ] Player-history-aware events
- [ ] Long-term mystery triggers

---

# Supporting Systems Summary

## Data Registry

Stores typed content definitions.

See `TECHNICAL_NOTES.md`.

## Content Validation

Ensures data references are valid and content does not silently break.

See `TECHNICAL_NOTES.md`.

## Save System

Stores player memory and persistent progression.

See `TECHNICAL_NOTES.md`.

## Presentation System

Handles UI, audio, VFX, feedback, atmosphere, and readability.

## Developer Tools

Debug and validation tools needed to keep a solo forever project maintainable.

See `TECHNICAL_NOTES.md`.

---

# Open System Questions

Use this section for unresolved design questions.

- [ ] Should relics use ammo, heat, cooldowns, or mixed mechanics?
- [ ] Should the player have shields, armor, wards, or only health?
- [ ] Should Anchors allow fast travel in the MVP?
- [ ] Should Cycle resets be voluntary, mandatory, or both?
- [ ] How much should the player see of the Run Contract?
- [ ] Should perk respec be cheap, expensive, or limited?

---

# Future System Additions

Add new proposed systems here before promoting them into the main design.

## Candidate System: Factions

Status: Unapproved

Purpose:

- Could allow enemy groups to interact.
- Could make strata feel more alive.
- Might increase complexity too early.

Decision:

- Do not build before Ash-Walk Pilgrimage.

## Candidate System: Crafting

Status: Unapproved

Purpose:

- Could allow relic refinement.
- Risk of distracting from found loot.

Decision:

- Avoid early. Prefer loot drops and simple rerolls first.

## Candidate System: Leaderboards

Status: Unapproved

Purpose:

- Could support long-term replayability.
- Not part of core identity.

Decision:

- Do not build before replayability spine is stable.
