# Cenotaph Identity Contract

## Purpose

This is the highest-level design guardrail for Cenotaph.

Read this before adding or changing systems, content, UI, economy, loot,
combat, art direction, level structure, or progression.

If another document or implementation idea conflicts with this contract, this
contract wins unless the project owner explicitly says otherwise.

---

## North Star

Cenotaph is a surreal, decayed-mountain RPG looter shooter about repeated
ascents through a hostile metaphysical ruin.

It should feel like:

- A pilgrimage up an impossible mountain.
- A dark fantasy shooter with ritual pressure.
- A Soulslike world of danger, consequence, and mystery.
- A loot RPG where weapons and relics feel cursed, named, strange, and useful.
- A progressive, dreamlike descent into symbolic logic, even while climbing.

It must not drift into:

- A Tarkov-style extraction shooter.
- A generic low-poly asset flip.
- A military survival raid game.
- A practical scavenging economy.
- A comedic Borderlands tone.
- A generic arena shooter with loot numbers attached.

The guiding question is:

```text
How far can the player ascend into the Cenotaph before the mountain takes
something from them?
```

Not:

```text
Can the player extract with valuables?
```

---

## Core Fantasy

The player is not a contractor, soldier, scavenger, or raider.

The player is a pilgrim, intruder, heir, weapon-bearer, or failed claimant
climbing through the corpse-memory of a mountain that should not exist.

The mountain is not a map. It is the central force of the game.

The mountain should feel:

- Ancient
- Vertical
- Hostile
- Sacred
- Decayed
- Royal
- Mechanical in impossible ways
- Beautiful in a damaged way
- Larger than the player can understand

The player climbs because staying below is impossible.

---

## Core Loop Language

Use this language when designing systems:

```text
Ascend -> survive ritual combat -> claim cursed build pieces -> reach an Anchor
-> change self or world -> ascend deeper
```

Avoid this language:

```text
Deploy -> loot -> extract -> sell -> restock -> repeat
```

Anchors are not extraction points. They are ritual checkpoints, memory stakes,
banking rites, or temporary claims against the mountain.

Runs are not raids. They are ascents, cycles, pilgrimages, attempts, echoes, or
coronation failures.

Loot is not vendor trash. Loot is memory, weaponry, temptation, survival, and
identity.

---

## Genre Hierarchy

When inspirations compete, use this order:

1. Cenotaph identity comes first.
2. Soulslike pressure shapes the world, danger, anchors, death, enemies, and
   mystery.
3. Looter shooter systems provide buildcraft, weapon variety, drops, rarity,
   affixes, and replayability.
4. Surreal progressive rock dream logic shapes tone, naming, space, memory,
   time, and symbolic weirdness.
5. Shooter RPG readability keeps combat playable and responsive.

Borderlands is useful for loot density and build variety, not comedy or tone.

Dark Souls is useful for pressure, myth, danger, and spaces that feel authored,
not for copying medieval fantasy.

King Crimson style influence is useful for dream logic, fractured time, ritual
decay, symbolic violence, and strange confidence, not direct reference or parody.

---

## Forbidden Drift

Do not add systems that make Cenotaph feel primarily like extraction survival.

Avoid:

- Extraction timers as the main pressure.
- Market, flea, trader, insured item, or raid-stash language.
- Practical military itemization.
- Generic scrap economies.
- UI that looks like tactical inventory management.
- Levels that feel like loot arenas with exits.
- Enemies that feel like armed squads or practical patrols.
- Low-poly style used as an excuse for weak silhouettes or empty spaces.
- AI-invented pseudo-production models that establish anatomy, ornament, or
  iconography without the developer's direction.
- Synthetic one-shot bleeps, chimes, bells, hit tones, and UI confirmations
  standing in for authored sound.

Survival pressure is allowed.

Resource loss is allowed.

Banking at Anchors is allowed.

But the emotional meaning must be pilgrimage, ritual risk, and ascent, not
shopping-run extraction.

---

## Visual Contract

The game may use low-poly or simple assets during development, but the final
direction is not "cheap low-poly."

The intended look is:

- Monumental silhouettes.
- Decayed royal architecture.
- Pale ash, black stone, tarnished gold, deep red, bone, iron, and sickly light.
- Angular figures that read clearly in combat.
- Impossible vertical spaces.
- Ritual machinery.
- Cathedrals, processional roads, throne fragments, chains, banners, ossuaries,
  observatories, broken instruments, and summit machinery.
- Strong material identity even when geometry is simple.

Every asset should answer:

```text
Why does this belong on the decayed mountain of the Cenotaph?
```

If the answer is only "the player needs an item/enemy/wall," redesign it.

---

## Solo Production Contract

Cenotaph is a solo-developed project. Its production model must favor systems,
reuse, validation, and strong art direction over large volumes of bespoke art.

The project should lean on:

- Programming-heavy systemic depth.
- Reusable level kits, materials, and animation roles.
- Simple geometry with deliberate composition, lighting, fog, color, scale, and
  sound.
- Data-driven enemies, relics, encounters, routes, events, and cycle mutations.
- Procedural or generated support for code, validation, placement, and data
  where it improves consistency. It must not invent visual or audio direction.
- Small authored spaces that imply a larger mountain through vertical framing,
  occlusion, landmarks, ambience, and route structure.

Avoid production plans that require:

- Hundreds of bespoke weapon models before loot is fun.
- Dozens of production enemy models before enemy roles are proven.
- Large cinematic sequences before the playable ascent loop works.
- Massive handcrafted maps before compact strata are replayable.
- Detail-heavy art that cannot be maintained by a solo developer.

The preferred production pattern is:

```text
Few strong reusable assets
+
many systemic combinations
+
strict validation
+
distinct presentation
```

Until visual direction is supplied, runtime placeholders must be honest single
primitives. Readability comes from dimensions, material role, motion, lighting,
and HUD state rather than invented anatomy or ornament. Silence is preferable to
synthetic one-shot feedback; semantic cue hooks wait for authored recordings.

---

## Loot Contract

Loot should feel like cursed buildcraft, not generic gear score.

Prefer:

- Named relics.
- Weapon families tied to mountain strata, dead courts, forgotten rites, and
  broken royal machinery.
- Rarity that changes behavior, not just numbers.
- Affixes with identity.
- Strange mechanical tradeoffs.
- Item descriptions that imply missing history.

Avoid:

- Plain military guns.
- Generic "common assault rifle" naming.
- Vendor-trash junk piles.
- Pure DPS upgrades without identity.
- Loot that exists only to be sold.

Good item direction:

```text
Starless Choir Rifle
The Red Minister's Fingerbone
Crown-Sealed Ash
Lark-Tongue Splinter
Debt of the Lower Procession
```

Bad item direction:

```text
Rusty SMG
Common Rifle
Scrap Metal
Vendor Junk
Extraction Token
```

---

## Enemy Contract

Enemies are inhabitants, consequences, wardens, failed pilgrims, animated rites,
or fragments of the mountain.

They should not feel like practical soldiers or generic mobs.

Each enemy needs:

- A readable combat role.
- A silhouette.
- A reason it belongs to a stratum.
- A ritual, royal, architectural, or memory-based identity.
- A counterplay pattern.

Elite enemies should be authored intentionally. Do not erase authored overrides
such as health, loot table, scale, or material identity during registry
materialization.

---

## Systems Contract

Future code should make the Cenotaph easier to extend without genre drift.

Prioritize systems that support:

- Stable runtime entity IDs.
- Runtime state separate from authored level data.
- Deterministic loot from stable source IDs.
- Save data for defeated enemies, collected pickups, spawned loot, opened
  routes, fired events, and changed world state.
- Ritualized Anchors and ascent progression.
- Weapon, relic, affix, status, and buildcraft expansion.
- Bosses with phases, arena state, and named drops.
- Level events that make the mountain feel reactive and haunted.

Be cautious with systems that primarily support:

- Extraction logistics.
- Stash markets.
- Tactical inventory realism.
- Commodity trading.
- Generic crafting.
- Feature work that does not affect ascents.

---

## Naming Rules

Use names that sound like they came from a dead symbolic kingdom, not a modern
loot economy.

Prefer words like:

- Anchor
- Ascent
- Stratum
- Relic
- Rite
- Cinder
- Choir
- Crown
- Debt
- Omission
- Procession
- Throne
- Pilgrim
- Remembrance
- Ash
- Wound
- Vessel
- Claim

Avoid words like:

- Extract
- Raid
- Stash
- Trader
- Market
- Insurance
- Scrap value
- Vendor junk
- Mission payout
- Loadout insurance

---

## Feature Acceptance Test

Before adding a feature, answer yes to at least three:

- Does it make the climb more tense, strange, readable, or meaningful?
- Does it make the mountain feel older, taller, more decayed, or more alive?
- Does it improve ritual combat, buildcraft, relic hunting, or ascent survival?
- Does it create a player decision inside an ascent?
- Does it create future content hooks without changing the core identity?
- Does it avoid extraction-shooter language and incentives?
- Does it strengthen the Cenotaph instead of just adding a genre feature?
- Can it be delivered with reusable assets, data-driven variation, or systemic
  depth appropriate for a solo developer?

If the feature fails this test, do not build it yet.

---

## Implementation Reminder For Coding Agents

When implementing requested changes:

1. Preserve the mountain-as-protagonist fantasy.
2. Prefer "ascent", "Anchor", "relic", "rite", and "stratum" framing.
3. Avoid extraction shooter concepts unless the user explicitly requests them.
4. Keep unresolved models as single neutral primitives; do not infer visual
   design from lore or names.
5. Keep loot strange, named, useful, and tied to the mountain.
6. Keep systems extensible for surreal RPG looter shooter content.
7. Prefer reusable content, systemic variation, and validation over bespoke
   asset volume.
8. If a requested change risks genre drift or solo-dev scope drift, call that
   out before implementing
   or offer a Cenotaph-aligned version.
9. Do not generate production-looking models or synthesized one-shot sounds
   unless the user explicitly reverses the primitive-and-silence policy.

The mountain is the protagonist. The player climbs because staying below is
impossible.
