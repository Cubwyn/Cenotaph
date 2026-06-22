# Cenotaph Technical Notes

## Purpose

This document tracks technical architecture, Rust direction, data structures, validation, saves, debugging, and long-term maintainability.

This is not final engine documentation.

It is the technical north star.

---

# Technical Goals

Cenotaph is a solo Rust forever project.

The architecture should prioritize:

- Strong typing
- Data-driven content
- Modular systems
- Content validation
- Deterministic generation where useful
- Debug tooling
- Long-term maintainability

---

# Architecture Mental Model

```text
cenotaph_core
  ascent
  relics
  builds
  threats
  cycles
  save
  registry

cenotaph_content
  strata
  relic_packs
  enemy_packs
  event_packs
  lore_packs

cenotaph_tools
  debug
  validators
  simulators
  test_maps

cenotaph_presentation
  ui
  audio
  vfx
  feedback
```

This does not need to be the literal crate structure at the start.

It is the mental model for keeping the project clean.

---

# Core Rust Rule

Use strong types for systems.

Use data files for content.

Use validation to connect them safely.

Recommended pattern:

```text
Content lives in data.
Data is validated at startup.
Runtime uses typed IDs.
```

Avoid hardcoding content everywhere.

Avoid runtime string lookups for core gameplay logic where typed IDs are safer.

---

# Typed ID Candidates

Examples:

```text
RelicArchetypeId
WeaponFamilyId
RelicPrefixId
RelicSuffixId
RelicTraitId
NamedRelicId
EnemyArchetypeId
EnemyModifierId
PerkId
StatusEffectId
StratumId
RouteId
EncounterTableId
HazardId
WorldEventId
LootTableId
CycleMutationId
LoreFragmentId
NpcId
BossId
```

---

# Data Registry System

## Purpose

The Data Registry stores definitions for game content.

It should allow the game to add content without rewriting core systems.

## Registered Content Types

```text
WeaponFamily
WeaponArchetype
RelicPrefix
RelicSuffix
RelicTrait
NamedRelic
EnemyArchetype
EnemyModifier
BossDefinition
PerkDefinition
StatusEffect
StratumDefinition
RouteDefinition
EncounterTable
HazardDefinition
WorldEvent
LootTable
CycleMutation
LoreFragment
NPCDefinition
```

## MVP Requirement

- Data-driven relic definitions
- Data-driven enemy definitions
- Data-driven loot tables

## Expansion

- Hot reload during development
- Content pack loading
- Validation reports
- Debug spawning by ID
- Internal mod-like structure

---

# Content Validation System

## Purpose

Validation protects the forever project from collapsing under its own content.

Validation should fail loudly.

Broken content should not silently produce broken runs.

## Validation Should Check

```text
Does every relic reference a valid archetype?
Does every loot table point to valid relics?
Does every enemy modifier support its target enemy?
Does every perk reference valid stats?
Does every status effect have valid behavior?
Does every stratum have encounter tables?
Does every cycle mutation have valid conditions?
Does every named relic have valid drop sources?
Does every route have at least one encounter or reward?
```

## MVP Requirement

- Startup validation for core content tables
- Clear error messages

## Expansion

- Full content validation reports
- Balance warnings
- Loot distribution checks
- Missing asset checks
- Broken reference detection
- Automated test runs

---

# Save System

## Theme

The player remembers what the mountain forgets.

## Save Data Should Store

- Player stats
- Perks
- Inventory
- Equipped relics
- Currencies
- Current Anchor
- Current Cycle
- Unlocked content
- Discovered lore
- Permanent world changes
- Collection history

## MVP Requirement

- Save inventory
- Save equipped relic
- Save Anchor
- Save basic progression

## Expansion

- Multiple profiles
- Build loadouts
- Cycle history
- Seed history
- Permanent world scars
- Relic archive
- Discovered mystery chains

---

# Run Contract Structure

Every ascent should have an internal run contract.

Example conceptual structure:

```text
RunContract
  cycle_number
  primary_stratum
  route_bias
  threat_bias
  relic_bias
  world_events
  active_mutations
  active_secrets
  persistent_memory_flags
```

## Purpose

The Run Contract gives each ascent an identity.

It prevents replayability from becoming random noise.

## MVP Requirement

- One run modifier
- One threat bias
- One loot variation

## Expansion

- Weighted selection
- Player-history-aware conditions
- Rare chains
- Cycle milestones
- Secret triggers

---

# Developer Tools

Developer tools are required, not optional.

They allow fast testing, balance checks, and safe expansion.

## Required Tools

- Spawn enemy
- Spawn relic
- Grant currency
- Grant perk points
- Set cycle number
- Teleport to test area
- Toggle god mode
- Print loot roll
- Inspect enemy stats
- Reload content data
- Trigger world event
- Trigger run contract

## MVP Requirement

- Debug test map
- Spawn relic command
- Spawn enemy command
- God mode
- Print current player stats

## Expansion

- Loot simulator
- Build simulator
- Cycle simulator
- Encounter test room
- Balance graphs
- Content validation UI
- Automated run summaries

---

# Presentation Technical Notes

Presentation includes:

- UI
- Audio
- VFX
- Feedback
- Atmosphere

## UI Requirements

- HUD
- Health
- Ammo or heat
- Pickup prompt
- Inventory
- Item tooltip
- Item comparison
- Perk screen
- Anchor menu
- Sanctuary menu
- Cycle modifier display

## Audio Requirements

- Weapon sounds
- Enemy sounds
- Hit feedback
- Loot pickup sounds
- Rarity chimes
- Ambient loops
- Suppression audio effects
- Dynamic combat intensity

## Visual Feedback Requirements

- Muzzle flash
- Impact effects
- Enemy hit reaction
- Loot glow
- Rarity colors
- Status effect visuals
- Suppression distortion
- Environmental particles

---

# Determinism Notes

Useful deterministic systems:

- Run contract generation
- Loot table testing
- Cycle mutation testing
- Encounter table simulation
- Save/load reproduction
- Debug seeds

Open questions:

- [ ] Should every ascent have a visible seed?
- [ ] Should the player be able to replay a previous seed?
- [ ] Should named relic drops be deterministic under certain conditions?
- [ ] Should persistent memory break determinism intentionally?

---

# Performance Notes

The mountain should feel huge without requiring everything to exist at once.

Future systems may need:

- Level chunking
- Enemy activation ranges
- Object pooling
- Projectile cleanup
- Save-safe unloading
- Visibility control
- Vertical streaming
- Background asset loading
- Performance debug overlay

MVP requirement:

- Small test area runs smoothly.
- Enemies deactivate or simplify when far away.
- Projectiles clean up properly.

---

# Technical Backlog

## Early

- [ ] Decide engine/framework.
- [ ] Define content data format.
- [ ] Create typed ID pattern.
- [ ] Create simple registry.
- [ ] Create simple validation pass.
- [ ] Create debug test map.
- [ ] Create spawn enemy command.
- [ ] Create spawn relic command.
- [ ] Create basic save format.

## Mid

- [ ] Hot reload content data.
- [ ] Loot simulation tool.
- [ ] Encounter simulation tool.
- [ ] Build stats inspector.
- [ ] Run contract inspector.
- [ ] Cycle mutation inspector.

## Late

- [ ] Internal content pack loader.
- [ ] Advanced validation reports.
- [ ] Automated run summaries.
- [ ] Performance profiling overlay.
- [ ] Asset dependency validation.
- [ ] Possible mod support.

---

# Open Technical Questions

- [ ] What Rust game engine or framework will be used?
- [ ] Should content files use RON, TOML, JSON, YAML, or custom format?
- [ ] Should save data use binary, RON, JSON, or another format?
- [ ] Should the content registry load everything at startup?
- [ ] Should debug tools exist in release builds behind a flag?
- [ ] Should procedural generation be deterministic by default?
- [ ] How should physics and recoil movement be implemented safely?
- [ ] How much of the game should be ECS-driven?
