# Cenotaph Gameplay Identity Reset

**Status: authoritative design contract.**

This document records the owner-supplied gameplay reset adopted on 2026-08-11.
It changes gameplay interpretation and priority. It does not rewrite the
established lore, symbolism, terminology, Strata, UI style, or visual contract.

## Fundamental game

Cenotaph is an authored surreal RPG campaign inside an enormous decaying
metaphysical mountain. Borderlands-style loot and buildcraft provide long-term
mechanical depth; authored world design provides environmental weight and
mystery; surrealism provides the emotional and atmospheric character.

The campaign remains authored. Replayability comes from systemic variation in
loot, equipment, builds, enemy composition, optional encounters, and endgame
challenges. “Infinite” does not authorize replacing the campaign with an
endless procedural map generator.

The seven Strata remain the established places:

1. Ash-Walk
2. Ward of Irons
3. Hanging Slums
4. Sanctuary
5. Gallery of Wind
6. Mirror-Crust
7. The Breach

Their identities are defined in [`CENOTAPH_STRATA.md`](CENOTAPH_STRATA.md).

## Primary loop

```text
EXPLORE
  -> ENCOUNTER
  -> COMBAT
  -> REWARD
  -> EVALUATE LOOT
  -> CHANGE EQUIPMENT OR BUILD
  -> BECOME MORE CAPABLE
  -> CONTINUE THROUGH THE CAMPAIGN
```

Exploration happens in memorable authored places. Combat is primarily shooter
combat and must stay readable and responsive. Rewards can include equipment,
relics, perks, progression, lore, resources, and access. Loot is valuable when
it creates a build decision, not merely because its number is larger.

The primary structure is not:

- enter run, loot, extract, sell, repeat;
- enter random dungeon, clear rooms, reset, repeat;
- cycle, reset the world, repeat; or
- survive until death and treat death as the campaign loop.

Anchors remain ritual checkpoints, not extraction points. Their exact resource,
death, and banking consequences must remain compatible with the campaign.

## System priorities

1. Authoritative lore
2. Cenotaph identity
3. World and Strata
4. Atmosphere
5. Loot and buildcraft
6. Combat
7. Character progression
8. Campaign progression
9. New Game+
10. Optional endgame
11. Convenience

Do not simplify a mechanic by changing a higher-priority element.

## Architecture consequences

### Keep

- Authored level geometry and Stratum identity.
- Data-driven content and strict cross-reference validation.
- Stable authored IDs for events, loot sources, Anchors, and named encounters.
- Transactional level/content preparation and staged save writes.
- Neutral single-primitive runtime placeholders until visual direction exists.
- Silent placeholder one-shot audio policy and non-tonal ambience.
- Relics as physical manifestations of memory and forgotten meaning.

### Modify

- The current Relic System must grow into a flexible equipment/loot system
  without requiring a unique model for every generated combination.
- Character progression must become distinct from equipment progression.
- Replayability must support authored campaign replay, New Game+, and optional
  endgame rather than treating every death as a new Cycle.
- Save data must eventually distinguish campaign state, character progression,
  equipment, discoveries, and any NG+ state.

### Demote or remove as primary structure

- Cycle reset as the main progression loop.
- Automatic Cycle advancement on death as an assumed final rule.
- Replacing authored campaign progression with procedural runs.
- Extraction, trader, market, insurance, stash, or vendor-trash incentives.

The current prototype still contains some of these assumptions. They are
identified below; they must not be expanded until their replacement is defined.

## UNDEFINED design decisions

These are intentionally not implemented by this reset:

- Weapon families, base behaviors, generated properties, modifiers, special
  effects, rarity rules, and comparison UI.
- Whether weapons and Relics are one equipment model or related item types.
- Character level, attributes, perk structure, permanent unlocks, and respec.
- Equipment slots, inventory capacity, duplicate handling, and item replacement.
- Ammo, heat, cooldown, shields, armor, wards, or other resource models.
- Death consequences, respawn state, resource loss, enemy/world reset, and loot
  recovery.
- Campaign completion state and the exact New Game+ boundary.
- Which equipment, Relics, perks, discoveries, currencies, quests, NPC states,
  or world changes persist into New Game+.
- Optional endgame difficulty, challenge, encounter, and reward rules.

For any of these, use this format before implementation:

```text
UNDEFINED DESIGN DECISION

Question:

Why this decision matters:

Current conflicting possibilities:

No implementation should proceed until this is defined.
```

## Current prototype conflict register

| Existing area | Current assumption | Classification | Required next step |
| --- | --- | --- | --- |
| `src/game/cycle.rs` | Cycle number selects modifiers and can advance | MODIFY / BLOCKED | Define whether any Cycle concept survives as optional endgame or lore |
| `src/core/engine/update.rs` | Death loses unsecured resource and advances Cycle | UNDEFINED | Define death consequences before changing or expanding it |
| `src/game/save.rs` | Save stores one cycle number and one current run state | MODIFY | Define campaign profile and NG+ persistence rules |
| `src/game/relic.rs` and `src/data/relic.rs` | Relics are fixed definitions with a few combat multipliers | MODIFY | Define equipment model, generation, rarity, and comparison behavior |
| `docs/design/SYSTEMS.md` | Cycle Director is a master system and reset loop | MODIFY | Replace with campaign replayability contract |
| `docs/design/ROADMAP.md` | Cycle reset is a milestone requirement | REMOVE | Replace with authored campaign completion and explicit NG+ design |
| `levels/*.json` | Authored spaces and stable events/loot sources | KEEP | Extend through validated content packs |

## Development rule

Read the identity, lore, Strata, content, and UI contracts before gameplay
changes. Inspect the existing architecture. Classify the affected system as
KEEP, MODIFY, REMOVE, or UNDEFINED. Identify conflicts. Only then design and
implement. Examples and genre conventions are not specifications.
