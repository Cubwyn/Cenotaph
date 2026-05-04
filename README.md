# Cenotaph: The Great Omission

> Version: 0.334

> Genre: Vertical Action RPG / Open-Zone Looter-Shooter

> Core Loop: Explore -> Loot -> Optimize -> Fight -> Repeat (Infinitely)

> Tone: Surreal decay, tragic beauty, chaotic exploration.

## Executive Summary

Cenotaph is a modular action RPG set inside a sunken mountain composed of crushed cathedrals, petrified roots, and ancient machinery. Players ascend seven massive, non-linear Strata, hunting for legendary weapons and optimizing unique character builds to survive the climb.

The game features **infinite New Game+ (NG+) replayability**. Upon reaching the summit or choosing to rest, the player can initiate a new cycle where the world regenerates with increased difficulty, altered enemy compositions, and entirely new loot pools. There is no "final" state; the goal is to discover every permutation of the world, master every build, and collect the most powerful artifacts the mountain can generate.

Death is a setback, not a reset. Players respawn at the last Sanctuary Anchor, retaining their Perks, unlocked blueprints, and equipped gear, but losing unsecured resources. The core fantasy is one of **limitless accumulation** and **mastery of chaos**.

### The Elevator Pitch
A vertical looter-shooter with infinite replayability. Every enemy, puzzle, and parkour challenge drops unique, randomized loot. Players can grind for the perfect "God Roll" weapon, master the vertical traversal, or solve the mountain's hidden puzzles. The world evolves with every cycle, offering endless variations of enemies, environments, and rewards.

## World Architecture

The game takes place within The Great Omission, a tectonic shaft of impossible depth.

### Key Locations
- The Pillar of Regret: The main play space—a mountain formed from compressed cities and machinery.
- The Obsidian Spire: A black glass tower piercing the cavern ceiling.
- The Sanctuary: The central safe hub where players manage inventory, upgrade perks, and fast travel.
- Anchor Chains: Colossal chains stabilizing the mountain, serving as key traversal points.

### The Seven Strata and Sub-Level Hierarchy

The following hierarchy outlines the primary Strata and their associated side areas, sub-levels, and hidden zones. These areas are designed to offer alternative routes, specific loot pools, and unique challenges.

- The Ash-Walk (Stratum 1: Collapse)
    - Primary Zone: Wagon Graveyard Expanse
        - Sub-Level: The Fossilized Root Caves
            - Hidden Zone: The Buried Wagon Cache (High-tier Scrap)
        - Sub-Level: The Dust Storm Corridors
            - Hidden Zone: The Silent Watcher's Outpost (Elite Camp)
    - Side Objective: The Wagon Graveyard Rescue
        - Reward: Unique "Ash-Drifter" Class Mod
    - Parkour Challenge: The Falling Wagon Run
        - Reward: "Gravity-Defier" Movement Trinket

- The Ward of Irons (Stratum 2: Industrial Insanity)
    - Primary Zone: The Bleeding Factory Floor
        - Sub-Level: The Rust-Slick Catwalks
            - Hidden Zone: The Ventilation Shaft Maze (Stealth Route)
        - Sub-Level: The Suspended Machinery Hangar
            - Hidden Zone: The Power Core Chamber (Boss: The Overheated Golem)
    - Side Objective: Restore the Steam Cycle
        - Reward: "Iron-Heart" Perk Node (Heat Resistance)
    - Puzzle: The Pressure Valve Sequence
        - Reward: "Steam-Press" Weapon Mod (Adds knockback)

- The Hanging Slums (Stratum 3: Precarious Life)
    - Primary Zone: The Cliffside Tenements
        - Sub-Level: The Rooftop Parkour Network
            - Hidden Zone: The Wire-Tangled Attic (Legendary Weapon Chest)
        - Sub-Level: The Basement Sewer Tunnels
            - Hidden Zone: The Forgotten Water Cistern (Resource Cache)
    - Side Objective: The Refugee Escort
        - Reward: "Shadow-Step" Movement Perk
    - Special Enemy: The Silent Stalker (Hidden in shadows)
        - Reward: "Whisper" Cloak (Temporary invisibility)

- The Sanctuary (Stratum 4: Broken Divinity)
    - Primary Zone: The Central Cathedral Hub
        - Sub-Level: The Vendor Atrium
            - Function: Weapon Rerolling, Blueprint Crafting, Perk Unlocks
        - Sub-Level: The Quest Board Archives
            - Function: Daily/Weekly Challenges, Lore Entries
        - Sub-Level: The Fast Travel Nexus
            - Function: Instant travel to unlocked Strata entrances
    - Special Feature: The Memory Shrine
        - Function: View collected lore, completed achievements, and NG+ stats

- The Gallery of Wind (Stratum 5: Breath of the Abyss)
    - Primary Zone: The Trumpet Carvings
        - Sub-Level: The Aerial Currents
            - Hidden Zone: The Floating Platform Garden (High-Speed Challenge)
        - Sub-Level: The Deep Wind Tunnels
            - Hidden Zone: The Wind Singer's Nest (Elite: The Gale Harpy)
    - Side Objective: Silence the Wind Singer
        - Reward: "Gale-Force" Projectile Speed Perk
    - Parkour Challenge: The Wind Tunnel Sprint
        - Reward: "Zephyr" Boots (Increased air control)

- The Mirror-Crust (Stratum 6: Reality Distortion)
    - Primary Zone: The Glass Forest
        - Sub-Level: The Reflection Pools
            - Hidden Zone: The Inverted Room (Puzzle: Reverse Gravity)
        - Sub-Level: The Shattered Pathways
            - Hidden Zone: The Memory Fragment Vault (Lore & Blueprints)
    - Side Objective: The Shattered Memory
        - Reward: "Mirror-Image" Deception Perk (Creates decoy)
    - Puzzle: The Prism Alignment
        - Reward: "Prism" Weapon Mod (Bullets split on impact)

- The Breach (Stratum 7: Geometric Collapse)
    - Primary Zone: The Fracturing Ascent
        - Sub-Level: The Elite Barracks
            - Hidden Zone: The Boss Rush Arena (Challenge Mode)
        - Sub-Level: The Void Threshold
            - Hidden Zone: The Final Memory (Secret Ending Trigger)
    - Side Objective: The Summit Push
        - Reward: Access to Ultra-Void Difficulty Tier
    - Special Boss: The Architect of Nothing
        - Reward: "Void-Caller" Legendary Weapon (Unique effect per NG+ cycle)

## Core Gameplay Systems

### 1. The Infinite Loop
- Explore: Traverse vertical zones with multiple routes (High/Mid/Low).
- Loot: Every enemy, chest, puzzle, and parkour challenge drops a randomized weapon or item.
- Optimize: Equip gear that synergizes with your active Perk Build.
- Fight: Engage in fast-paced FPS combat using vertical mobility.
- Repeat: Upon death, respawn at the last Anchor. Lose unsecured resources, but retain Perks and equipped gear.
- **New Game+**: Upon completing the summit or resting, the player can restart the cycle. The world regenerates with:
    - Increased enemy health and damage.
    - New enemy variants and modifiers.
    - Altered loot pools (new prefixes, suffixes, and weapon types).
    - Modified environmental hazards and puzzle configurations.

### 2. Modular Relic Weapons (The Loot Engine)
Weapons are not assembled; they are found. Every drop is a fully formed entity with randomized attributes.

- Weapon Classes:
    - Schizoid: Chaotic spread, high volatility.
    - Moonchild: Homing projectiles, tracking.
    - Sovereign: Heavy hitscan, high impact.
    - Hybrids: Combinations of the above.
- Randomization Layers:
    - Prefixes: "The Weeping" (Life Steal), "The Void" (Armor Ignore), "The Fractured" (Chance to shatter).
    - Suffixes: "of the Burning" (Fire Dmg), "of the Frost" (Slow), "of the Void" (Teleport on hit).
    - Stats: Fire Rate, Damage, Spread, Heat Gen, Reload Speed.
    - Elements: Fire, Ice, Shock, Void, Gravity, Time.
    - Special Effects: Bouncing Bullets, Explosive Rounds, Critical Multipliers, Life Steal, Ammo Refund.
- Rarity Tiers: Common (White) -> Rare (Green) -> Epic (Blue) -> Legendary (Purple) -> Mythic (Orange) -> **Transcendent (Rainbow)**.
- **Infinite Variety**: The loot pool expands with each NG+ cycle, introducing new combinations and effects.

### 3. Deep Perk Builds (Synergy System)
Players unlock Perk Points to invest in three distinct Skill Trees. Perks modify weapon behavior, movement, and survival, allowing for deep build customization.

- Tree A: The Ascendant (Movement & Verticality)
    - Focus: Wall-run speed, double jump height, recoil dash distance.
    - Synergy Example: "Recoil Boost" perk + High-Recoil Weapon = Infinite Air Time.
- Tree B: The Artificer (Combat & Weaponry)
    - Focus: Crit chance, heat capacity, reload speed, elemental damage.
    - Synergy Example: "Overheat Ignition" perk + High Heat Weapon = Area-of-Effect Explosion on Overheat.
- Tree C: The Survivor (Defense & Utility)
    - Focus: Max HP, damage resistance, loot magnet, revive chance.
    - Synergy Example: "Blood Thirst" perk + Life Steal Weapon = Infinite Sustain.

Build Philosophy: A "God Roll" weapon is useless without the matching Perks, and Perks are wasted without the right weapon. The grind is about finding the perfect synergy.

### 4. Vertical Traversal Mechanics
Movement is a tactical weapon.
- Recoil Dash: Shooting pushes the player; aiming down propels upward.
- Drop-Kick: Falling onto enemies deals massive damage and bounces the player up.
- Chain Swinging & Wall-Running: Essential for navigating the Strata.
- Design Principle: The world rewards height. Every encounter has a high, mid, and low approach.
- **Parkour Rewards**: Completing specific traversal challenges (e.g., "No Touch" runs, speed runs) awards unique trinkets and weapon mods.

### 5. Puzzle & Challenge Rewards
- **Environmental Puzzles**: Solving puzzles (e.g., gravity reversal, pressure valves) rewards unique weapon mods or blueprints.
- **Special Enemies**: Defeating hidden or elite enemies (e.g., "The Silent Stalker") rewards unique class mods or trinkets.
- **Bosses**: Defeating bosses rewards legendary weapons with unique effects that change per NG+ cycle.

## Enemy Factions & AI

Enemies are designed to challenge specific build types and encourage verticality.

- The Burdened: Stone-carrying husks. High physical impact, slow but tanky.
- The Silencers: Tall figures with sewn mouths. Suppress player abilities (Silence mechanic).
- The Paranoiacs: Cage-headed screamers. Area denial; alert entire zones if not killed quickly.
- The Harpies: Decayed marble angels. Agile aerial threats, require vertical counter-play.

- Elites: Randomly generated variants with unique modifiers (e.g., "Fire Immune," "Speed Boost").
- Bosses: Represent failed memories of the kingdom; guard major loot caches.
- **NG+ Variants**: In higher cycles, enemies gain new abilities, resistances, and drop rarer loot.

## Progression & Economy

### Currency
- Scrap: Dropped by all enemies. Used for purchasing ammo, medkits, and rerolling weapon stats at vendors.
- Memory Shards: Dropped by Elites/Bosses. Used to unlock new Perks and Craft Blueprints.
- **Cycle Tokens**: Earned by completing NG+ cycles. Used to unlock permanent global upgrades (e.g., increased drop rates, new weapon types).

### Difficulty Scaling (Strata Tiers)
Instead of a global "Madness Level," difficulty scales per Stratum and per NG+ cycle:
1. Standard: Normal stats, standard loot.
2. Hardened: 2x Enemy HP, increased Rare drop rate.
3. Nightmare: Elemental resistances, increased Legendary drop rate.
4. Ultra-Void: Unique enemy modifiers, Mythic drop rate.
5. **Transcendent**: (NG+ 10+) Enemies have unique abilities, drop Transcendent loot.

Players can replay Strata in higher tiers to grind for better gear.

## Visual & Audio Identity

### Visuals
- Textures: Cracked marble, tarnished gold, rusted iron, obsidian glass.
- Lighting: Shafted vertical beams, volumetric fog, high contrast near the peak.
- Silhouette: Everything points upward. Spires, chains, angular machinery fused with organic curves.
- Loot Feedback: Weapons glow with rarity colors; Legendaries have unique particle effects.
- **NG+ Visuals**: Each cycle introduces subtle visual changes (e.g., color shifts, new environmental effects) to signify progression.

### Audio
- Music: Industrial cathedral ambience, distant choral fragments, metallic percussion.
- SFX: Deep echo reverb, groaning chains, wind howling vertically.
- Feedback: Distinct "chimes" for loot rarity; unique "jam" sounds for overheating.
- **Dynamic Audio**: Music intensity scales with enemy density and NG+ cycle number.

## Narrative Structure

The story is told through environmental decay and the player's pursuit of the "Perfect Loadout."

- Act I: Discovery. The player learns the mechanics, explores the lower Strata, and understands the threat.
- Act II: Escalation. Reality destabilizes. Enemies become more aggressive. The player must optimize their build to survive.
- Act III: Resolution. The final ascent. The player reaches the summit with their ultimate build.
- Finale: The revelation that the surface is a dead universe. The choice to rest (end save) or continue the climb (New Game+).
- **Infinite Narrative**: Each NG+ cycle reveals new lore entries, hidden dialogues, and environmental storytelling elements, encouraging players to explore every corner of the world.

## Development Philosophy

### Thematic Pillars
1. The Great Omission: Reality is missing something. Loot is the only tangible memory.
2. Vertical Regret: Height equals proximity to truth. The climb is the only way forward.
3. Memory vs. Illusion: Perks are the only "truth" retained; the rest is a test of skill.
4. Beauty in Ruin: Decayed elegance, not grimdark brutality.
5. **Infinite Possibility**: The world is endless, and every cycle offers new discoveries.

### Quality Control
- No Generic Mechanics: If a mechanic doesn't serve exploration, loot, or verticality, it is cut.
- Thematic Consistency: Every system must reinforce the feeling of ascending a dying world.
- Multiple Paths: Every zone must have at least 3 distinct routes (High, Mid, Low).
- **Endless Content**: Ensure that every NG+ cycle introduces meaningful changes to gameplay, loot, and environment.

### Technical Standards
- Performance: Optimized for large vertical spaces and dynamic lighting.
- Extensibility: Modular code architecture for loot generation and procedural events.
- Cross-Platform: Designed for PC and Console compatibility.
- **Scalability**: The game must support hundreds of NG+ cycles without performance degradation.

## Future Roadmap (High Level)

- Phase 1: The Tight Core – Engine, Physics, Basic Movement, Loot Generation.
- Phase 2: Open-Zone & Content – 7 Strata, Enemy AI, Perk Trees, Vendors.
- Phase 3: Polish & Scale – Boss Encounters, Audio Integration, Endgame Modes, Leaderboards.
- Phase 4: Infinite Ascent – NG+ system, expanded loot pools, dynamic world changes, hidden content.

Note: This document serves as the living design bible. Specific implementation details may evolve, but the core loop and thematic pillars remain constant. The goal is to create a game that players can play forever, discovering new things with every cycle.
