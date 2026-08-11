# Cenotaph Content Guide

## Purpose

This document defines how to add new content to Cenotaph without breaking the project identity.

Use this file when designing:

- Strata
- Routes
- Relics
- Enemies
- Enemy modifiers
- Bosses
- Hazards
- World events
- Cycle mutations
- Lore fragments
- Content packs

Every content addition should support at least one of the following:

1. The climb
2. Survival
3. Relic hunting
4. Buildcraft
5. Replayability
6. Mystery

Content should also respect the solo production model. Prefer additions that can
be built from reusable assets, data-driven variants, systemic rules, and strong
presentation rather than large amounts of one-off art.

---

# Content Pack Philosophy

Cenotaph is a forever project.

Content should be added in packs when possible.

A content pack is a self-contained group of additions that can be tested, validated, expanded, and documented.

---

## Pack Types

### Stratum Pack

Contains:

- Rooms
- Routes
- Hazards
- Encounters
- Relic tables
- Events
- Secrets
- Lore fragments
- Boss or major objective

### Relic Pack

Contains:

- Weapon archetypes
- Prefixes
- Suffixes
- Traits
- Named relics
- Drop rules
- Flavor text

### Enemy Pack

Contains:

- Enemy archetype
- AI behavior
- Modifiers
- Loot tables
- Sounds
- Visual identity
- Counterplay rules

### Cycle Pack

Contains:

- Mutations
- Scaling rules
- Event rules
- Persistent changes
- New run contract options

### Lore Pack

Contains:

- Relic memories
- NPC dialogue
- Environmental text
- Dream fragments
- Contradictory records
- Hidden story chains

---

# Universal Content Checklist

Before adding content, answer:

- [ ] What system does this content support?
- [ ] What player decision does it create?
- [ ] What makes it replayable?
- [ ] What makes it readable?
- [ ] What makes it Cenotaph-specific?
- [ ] What is the counterplay?
- [ ] What can be expanded later?
- [ ] Does it work inside an ascent?
- [ ] Can it be built, tested, and maintained with solo-dev asset constraints?
- [ ] Does it reuse or extend existing systems, silhouettes, materials, or
      content rules?

---

# Stratum Template

Use this template for every Stratum.

## Stratum Name

### Theme

Describe the emotional and symbolic role.

### Visual Identity

List key materials, shapes, lighting, silhouettes, and landmarks.

### Gameplay Focus

Define what the Stratum teaches, tests, or emphasizes.

### Primary Threats

List common enemies and elite variants.

### Route Style

Describe High, Mid, and Deep route behavior.

### Hazards

List environmental dangers.

### Relic Bias

Define which relic families, traits, or affixes are more likely here.

### Cycle Mutation Ideas

List ways this Stratum can change between Cycles.

### Development Purpose

Explain why this Stratum exists in production terms.

### Future Additions

- [ ] New room type
- [ ] New hazard
- [ ] New enemy variant
- [ ] New relic table
- [ ] New event
- [ ] New lore fragment
- [ ] New shortcut
- [ ] New boss or elite encounter

---

# Stratum Documents

## Stratum 1: Ash-Walk

### Theme

Collapse, ash, and first survival.

The player sees the scale of the mountain and understands that the climb will not be gentle.

### Visual Identity

- Buried roads
- Broken shrines
- Ash fields
- Collapsed settlements
- Low ruins
- Distant vertical landmarks

### Gameplay Focus

- Basic combat
- Early route choice
- First relic hunting
- First elite encounter
- Basic survival pressure

### Primary Threats

- Ashbound
- Burdened
- Censer
- Basic Harpies
- Early elite variants

### Route Style

- Open routes
- Ruin paths
- Cave shortcuts
- Exposed high paths

### Hazards

- Ash storms
- Collapsing ruins
- Poor visibility

### Relic Bias

- Basic Sovereign relics
- Early survival relics
- Introductory affixes

### Cycle Mutation Ideas

- Ash storms become more frequent
- Harpies appear earlier
- Hidden caches become buried or revealed
- Burdened patrols change routes

### Development Purpose

Ash-Walk should become the first complete playable Stratum.

It teaches the player how to climb, fight, find relics, and survive.

### Future Additions

- [x] First named elite: The Ash-Warden, Bearer of the Last Chain
- [ ] First Ash-Walk boss
- [ ] Buried relic cache event
- [ ] Ash storm run contract
- [ ] Hidden pilgrim corpse lore chain

---

## Stratum 2: Ward of Irons

### Theme

Industrial confinement and machinery without purpose.

The player feels trapped inside a system that continues operating after meaning has died.

### Visual Identity

- Rusted machinery
- Pipes
- Suspended platforms
- Steam vents
- Grinding gears
- Iron corridors
- Vertical shafts

### Gameplay Focus

- Environmental pressure
- Tighter combat spaces
- Hazard awareness
- Machinery-based traversal

### Primary Threats

- Burdened
- Bellworn
- Silencers
- Armored enemy variants

### Route Style

- Corridors
- Machinery rooms
- Vertical industrial shafts
- Maintenance tunnels

### Hazards

- Steam bursts
- Crushing machinery
- Moving platforms
- Heat zones

### Relic Bias

- Heavy Sovereign relics
- Heat-based relics
- Armor-piercing effects

### Cycle Mutation Ideas

- Machinery timing changes
- Silence fields appear near machines
- Steam hazards become elemental
- Maintenance routes open or close

### Development Purpose

Ward of Irons introduces confined pressure and environmental danger.

### Future Additions

- [ ] Steam valve event
- [ ] Machine altar relic cache
- [ ] First major Silencer encounter
- [ ] Heat-based named relic
- [ ] Moving machinery route puzzle

---

## Stratum 3: Hanging Slums

### Theme

Precarious life above oblivion.

The player sees that people once tried to live inside the climb.

### Visual Identity

- Hanging homes
- Rope bridges
- Chain networks
- Cliffside structures
- Lanterns
- Broken lifts
- Deep drops

### Gameplay Focus

- Route risk
- Vertical navigation
- Ambushes
- Exploration rewards
- Optional hidden dwellings

### Primary Threats

- Harpies
- Paranoiacs
- Chainrunners
- Fast enemy variants

### Route Style

- Suspended paths
- Chain crossings
- Cliff routes
- Hidden residential interiors

### Hazards

- Falling structures
- Weak bridges
- Wind bursts
- Alarm escalation

### Relic Bias

- Mobility-support relics
- Moonchild tracking relics
- Route discovery relics

### Cycle Mutation Ideas

- Bridges collapse differently
- Paranoiacs appear in hidden dwellings
- Harpy nests migrate
- Some homes appear occupied again

### Development Purpose

Hanging Slums teaches risk/reward route selection and vertical danger.

### Future Additions

- [ ] Hidden dwelling event
- [ ] Harpy nest bounty
- [ ] Paranoiac alarm chain
- [ ] Collapsing bridge route
- [ ] Hanging merchant NPC

---

## Stratum 4: Sanctuary

### Theme

Temporary safety, melancholy, and preparation.

The Sanctuary is the only place that does not resist the player.

### Visual Identity

- Suspended cathedral
- Dim candles
- Tarnished gold
- Quiet pilgrims
- Broken statues
- Storage alcoves
- Anchor light

### Gameplay Focus

- Recovery
- Preparation
- Inventory management
- Perk allocation
- NPC interaction
- Cycle transition

### Primary Threats

None during normal gameplay.

The absence of threats is the point.

### Route Style

- Central hub
- Connected exits
- Return paths
- Locked or revealed chambers

### Hazards

None in normal state.

Rare cycle mutations may alter the Sanctuary, but it should remain functionally safe unless a major story or cycle event changes that rule.

### Relic Bias

- Vendors
- Storage
- Archive systems
- Relic comparison
- Possible rare Sanctuary-specific relics

### Cycle Mutation Ideas

- NPC dialogue changes
- New doors appear
- Statues move between cycles
- Vendors remember impossible events
- A dead pilgrim appears before they die elsewhere

### Development Purpose

Sanctuary gives the player relief and provides the practical hub for long-term progression.

### Future Additions

- [ ] Perk altar
- [ ] Relic archive
- [ ] Cycle altar
- [ ] Impossible pilgrim NPC
- [ ] Hidden Sanctuary chamber

---

## Stratum 5: Gallery of Wind

### Theme

Exposure, height, and insignificance.

The player is surrounded by open space and impossible currents.

### Visual Identity

- Vast chambers
- Floating ruins
- Wind streams
- Broken bridges
- Vertical light shafts
- Distant suspended islands

### Gameplay Focus

- Open vertical combat
- Aerial threats
- Route commitment
- Wind-based traversal
- Long sightlines

### Primary Threats

- Harpies
- Aerial elites
- Long-range enemies
- Wind-displaced enemies

### Route Style

- High exposure paths
- Floating platforms
- Wind-assisted routes
- Risky shortcuts

### Hazards

- Wind currents
- Falling risk
- Sudden gusts
- Visibility shifts

### Relic Bias

- Moonchild relics
- Anti-air relics
- Recoil-control relics
- Airborne combat effects

### Cycle Mutation Ideas

- Wind directions change
- Floating structures move
- Harpies gain modifiers
- Rare air routes appear

### Development Purpose

Gallery of Wind expands verticality without turning the game into a pure movement shooter.

### Future Additions

- [ ] Wind route variant
- [ ] Floating relic cache
- [ ] Aerial elite
- [ ] Anti-air named relic
- [ ] Storm cycle event

---

## Stratum 6: Mirror-Crust

### Theme

Reality distortion, reflection, and uncertainty.

The player should question what is real while still understanding what is mechanically happening.

### Visual Identity

- Reflective surfaces
- Glass terrain
- Duplicated architecture
- Inverted spaces
- Pale light
- Repeated silhouettes
- Rooms that seem remembered rather than built

### Gameplay Focus

- Hidden routes
- Ambushes
- Spatial uncertainty
- Reflection-based secrets
- Advanced enemy modifiers

### Primary Threats

- Silencers
- Mirror enemies
- Reality-altered elites
- Ambush variants

### Route Style

- Misleading paths
- Reflected shortcuts
- Hidden spaces
- Backtracking with changed context

### Hazards

- Mirror illusions
- False floors
- Gravity distortions
- Delayed enemy reflections

### Relic Bias

- Omission relics
- Void relics
- Reflection effects
- Duplicate projectile effects

### Cycle Mutation Ideas

- Rooms appear in different order
- Dead enemies leave reflections
- Secret caches only appear in mirrors
- Boss memories begin leaking into normal routes

### Development Purpose

Mirror-Crust introduces late-game surreal complexity while preserving readable mechanics.

### Future Additions

- [ ] Mirror cache
- [ ] Reflection elite
- [ ] False route event
- [ ] Omission named relic
- [ ] Gravity distortion hazard

---

## Stratum 7: The Breach

### Theme

Collapse, final ascent, and reality failing to hold its shape.

The player reaches the place where the mountain can no longer pretend to be whole.

### Visual Identity

- Fractured geometry
- Broken sky
- Floating cathedral pieces
- Impossible angles
- Void light
- Split terrain
- Reality scars

### Gameplay Focus

- Endgame combat
- Boss encounters
- High-risk traversal
- Cycle completion
- Major relic rewards
- Secret endings or transitions

### Primary Threats

- Elite variants
- Bosses
- Mixed enemy groups
- Cycle-mutated enemies

### Route Style

- Dangerous vertical routes
- Boss gates
- Fragmented paths
- Hidden final shortcuts

### Hazards

- Void fractures
- Geometry collapse
- Suppression waves
- Reality shifts

### Relic Bias

- Mythic relics
- Transcendent relics
- Cycle-exclusive relics
- Build-defining named relics

### Cycle Mutation Ideas

- Bosses remember prior cycles
- The summit changes
- New endings become available
- Previous strata leak into the Breach
- Relics mutate after completion

### Development Purpose

The Breach provides the endgame ascent and transition into future Cycles.

### Future Additions

- [ ] First Breach boss
- [ ] Summit transition
- [ ] Cycle completion ritual
- [ ] Transcendent relic table
- [ ] Reality collapse event

---

# Relic Template

Use this when designing new relics.

Every implemented relic lives in `data/relics/*.toml` and owns its `id`,
`display_name`, `pickup_asset`, family, rarity, combat multipliers, hit-stun
bonus, and flavor text. `pickup_asset` is relative to `assets/` and may reuse an
existing `.obj`, `.gltf`, or `.glb` silhouette. Adding a relic must not require
another hard-coded Rust asset match.

## Relic Name

### Family

Sovereign / Moonchild / Schizoid / Other

### Rarity

Common / Rare / Epic / Legendary / Mythic / Transcendent

### Mechanical Identity

What does this relic do?

### Build Role

What kind of build wants this relic?

### Drawback or Tension

What prevents it from being universally best?

### Drop Source

Where does it come from?

### Cycle Behavior

Does it change in later Cycles?

### Flavor Text

One to three lines.

### Future Variants

- [ ] Prefix variant
- [ ] Cycle-mutated variant
- [ ] Boss-specific variant
- [ ] Corrupted variant

---

# Enemy Template

Use this when designing new enemies.

## Enemy Name

### Role

Grunt / Tank / Glass Cannon / High-Speed / Aerial / Ranged Harasser /
Suppressor / Escalator / Swarmer / Controller / Duelist / Boss / Other

### Theme

What does this enemy represent?

### Behavior

How does it move, attack, and react?

### Player Counterplay

How should the player respond?

### Encounter Use

What group composition, room shape, or route problem does this enemy support?

### Threatened Player Options

What does this enemy deny or pressure?

### Drops

What does it reward?

### Compatible Modifiers

List enemy modifiers that can apply.

### Future Variants

- [ ] Elite version
- [ ] Cycle version
- [ ] Boss version
- [ ] Stratum-specific variant

---

# World Event Template

## Event Name

### Event Type

Danger / Reward / Mystery / Route / Combat / Sanctuary / Cycle

### Trigger

When can this occur?

### Effect

What changes mechanically?

### Player Decision

What choice does this create?

### Reward or Consequence

What can the player gain or lose?

### Strata

Where can it appear?

### Cycle Behavior

How can it change over time?

### Future Additions

- [ ] Rare version
- [ ] Connected lore fragment
- [ ] Enemy variant
- [ ] Relic interaction

---

# Cycle Mutation Template

Cycle mutations are optional lore, world-state, or post-campaign endgame
content. They are not permission to make Cycle reset the primary progression
structure. Any implemented mutation requires an explicit campaign/NG+ design
decision first.

## Mutation Name

### Theme

What changed in the mountain?

### Mechanical Effect

What changes in gameplay?

### Affected Systems

- Ascent
- Relics
- Builds
- Threats
- Routes
- Sanctuary
- Lore

### Player Impact

What should the player notice?

### Counterplay

How can the player adapt?

### Unlock Condition

When can this mutation appear?

### Future Additions

- [ ] Stronger version
- [ ] Rare linked event
- [ ] Named relic interaction
- [ ] Boss interaction

---

# Content Backlog

Use this section to collect future content ideas before approving them.

## Relics

- [ ] The Hollow Choir
- [ ] Regret Engine
- [ ] The Widowmaker
- [ ] Mourning Bell
- [ ] The Unsaid

## Enemies

- [ ] Ashbound
- [ ] Censer
- [ ] Chainrunner
- [ ] Bellworn
- [ ] Anchor parasite
- [ ] Chain pilgrim
- [ ] Mirror of the player
- [ ] Bell-headed elite
- [ ] Root-machine hybrid

## Events

- [ ] Ash storm reveals relic cache
- [ ] Dead pilgrim appears before death
- [ ] Sanctuary statue moves
- [ ] Chain route collapses
- [ ] Boss memory leaks into lower Stratum

## Cycle Mutations

- [ ] Ash Memory
- [ ] Silence Below
- [ ] Mirror Descent
- [ ] Harpy Migration
- [ ] Forgotten Anchor

---

# Approval Status

Use this section to separate ideas from committed content.

## Approved For MVP

- [ ] One weapon family
- [ ] One baseline enemy role, preferably Ashbound or Burdened
- [ ] One elite modifier
- [ ] One hazard
- [ ] One safe room
- [ ] One Anchor
- [ ] One run modifier

## Approved For Ash-Walk Pilgrimage

- [ ] Three weapon families
- [ ] Ashbound
- [ ] Burdened
- [ ] Censer
- [ ] Harpy
- [ ] Chainrunner or Paranoiac
- [ ] Ash storm event
- [ ] Basic Sanctuary
- [ ] One boss or named elite

## Not Approved Yet

- [ ] Full crafting
- [ ] Multiplayer
- [ ] Full procedural terrain
- [ ] Seven complete Strata
- [ ] Complex NPC schedules
- [ ] Online features
