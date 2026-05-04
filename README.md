# Cenotaph: The Great Omission

> Version: 1.1 (Revised)
> Genre: Vertical Action RPG / Open-Zone Looter-Shooter
> Core Loop: Explore -> Loot -> Optimize -> Fight -> Repeat
> Tone: Surreal decay, tragic beauty, chaotic exploration.

## Executive Summary

Cenotaph is a modular action RPG set inside a sunken mountain composed of crushed cathedrals, petrified roots, and ancient machinery. Players ascend seven massive, non-linear Strata, hunting for legendary weapons and optimizing unique character builds to survive the climb.

Unlike traditional roguelikes, the world does not reset on death. Instead, players face grounded consequences: dropping loot, losing resources, and respawning at the last Sanctuary Anchor. The core fantasy is one of persistent progression—building a "God Roll" loadout through relentless grinding, strategic perk allocation, and mastering vertical traversal mechanics.

### The Elevator Pitch
A vertical looter-shooter where every enemy drops a unique, fully-formed weapon. Players explore a dying world, craft synergistic builds using a deep perk system, and choose their own route to the summit. Death is a setback, not a reset, driving a cycle of exploration, optimization, and conquest.

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

- The Ward of Irons (Stratum 2: Industrial Insanity)
    - Primary Zone: The Bleeding Factory Floor
        - Sub-Level: The Rust-Slick Catwalks
            - Hidden Zone: The Ventilation Shaft Maze (Stealth Route)
        - Sub-Level: The Suspended Machinery Hangar
            - Hidden Zone: The Power Core Chamber (Boss: The Overheated Golem)
    - Side Objective: Restore the Steam Cycle
        - Reward: "Iron-Heart" Perk Node (Heat Resistance)

- The Hanging Slums (Stratum 3: Precarious Life)
    - Primary Zone: The Cliffside Tenements
        - Sub-Level: The Rooftop Parkour Network
            - Hidden Zone: The Wire-Tangled Attic (Legendary Weapon Chest)
        - Sub-Level: The Basement Sewer Tunnels
            - Hidden Zone: The Forgotten Water Cistern (Resource Cache)
    - Side Objective: The Refugee Escort
        - Reward: "Shadow-Step" Movement Perk

- The Sanctuary (Stratum 4: Broken Divinity)
    - Primary Zone: The Central Cathedral Hub
        - Sub-Level: The Vendor Atrium
            - Function: Weapon Rerolling, Blueprint Crafting, Perk Unlocks
        - Sub-Level: The Quest Board Archives
            - Function: Daily/Weekly Challenges, Lore Entries
        - Sub-Level: The Fast Travel Nexus
            - Function: Instant travel to unlocked Strata entrances
    - Special Feature: The Memory Shrine
        - Function: View collected lore and completed achievements

- The Gallery of Wind (Stratum 5: Breath of the Abyss)
    - Primary Zone: The Trumpet Carvings
        - Sub-Level: The Aerial Currents
            - Hidden Zone: The Floating Platform Garden (High-Speed Challenge)
        - Sub-Level: The Deep Wind Tunnels
            - Hidden Zone: The Wind Singer's Nest (Elite: The Gale Harpy)
    - Side Objective: Silence the Wind Singer
        - Reward: "Gale-Force" Projectile Speed Perk

- The Mirror-Crust (Stratum 6: Reality Distortion)
    - Primary Zone: The Glass Forest
        - Sub-Level: The Reflection Pools
            - Hidden Zone: The Inverted Room (Puzzle: Reverse Gravity)
        - Sub-Level: The Shattered Pathways
            - Hidden Zone: The Memory Fragment Vault (Lore & Blueprints)
    - Side Objective: The Shattered Memory
        - Reward: "Mirror-Image" Deception Perk (Creates decoy)

- The Breach (Stratum 7: Geometric Collapse)
    - Primary Zone: The Fracturing Ascent
        - Sub-Level: The Elite Barracks
            - Hidden Zone: The Boss Rush Arena (Challenge Mode)
        - Sub-Level: The Void Threshold
            - Hidden Zone: The Final Memory (Secret Ending Trigger)
    - Side Objective: The Summit Push
        - Reward: Access to Ultra-Void Difficulty Tier

## Core Gameplay Systems

### 1. The Grounded Loop
- Explore: Traverse vertical zones with multiple routes (High/Mid/Low).
- Loot: Every enemy and chest drops a randomized weapon with unique stats, elements, and effects.
- Optimize: Equip gear that synergizes with your active Perk Build.
- Fight: Engage in fast-paced FPS combat using vertical mobility.
- Repeat: Upon death, respawn at the last Anchor. Lose unsecured resources, but retain Perks and equipped gear.

### 2. Modular Relic Weapons (The Loot Engine)
Weapons are not assembled; they are found. Every drop is a fully formed entity with randomized attributes.

- Weapon Classes:
    - Schizoid: Chaotic spread, high volatility.
    - Moonchild: Homing projectiles, tracking.
    - Sovereign: Heavy hitscan, high impact.
    - Hybrids: Combinations of the above.
- Randomization Layers:
    - Prefixes: "The Weeping" (Life Steal), "The Void" (Armor Ignore).
    - Suffixes: "of the Burning" (Fire Dmg), "of the Frost" (Slow).
    - Stats: Fire Rate, Damage, Spread, Heat Gen, Reload Speed.
    - Elements: Fire, Ice, Shock, Void.
    - Special Effects: Bouncing Bullets, Explosive Rounds, Critical Multipliers.
- Rarity Tiers: Common (White) -> Rare (Green) -> Epic (Blue) -> Legendary (Purple) -> Mythic (Orange).

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

## Enemy Factions & AI

Enemies are designed to challenge specific build types and encourage verticality.

- The Burdened: Stone-carrying husks. High physical impact, slow but tanky.
- The Silencers: Tall figures with sewn mouths. Suppress player abilities (Silence mechanic).
- The Paranoiacs: Cage-headed screamers. Area denial; alert entire zones if not killed quickly.
- The Harpies: Decayed marble angels. Agile aerial threats, require vertical counter-play.

- Elites: Randomly generated variants with unique modifiers (e.g., "Fire Immune," "Speed Boost").
- Bosses: Represent failed memories of the kingdom; guard major loot caches.

## Progression & Economy

### Currency
- Scrap: Dropped by all enemies. Used for purchasing ammo, medkits, and rerolling weapon stats at vendors.
- Memory Shards: Dropped by Elites/Bosses. Used to unlock new Perks and Craft Blueprints.

### Difficulty Scaling (Strata Tiers)
Instead of a global "Madness Level," difficulty scales per Stratum:
1. Standard: Normal stats, standard loot.
2. Hardened: 2x Enemy HP, increased Rare drop rate.
3. Nightmare: Elemental resistances, increased Legendary drop rate.
4. Ultra-Void: Unique enemy modifiers, Mythic drop rate.

Players can replay Strata in higher tiers to grind for better gear.

## Visual & Audio Identity

### Visuals
- Textures: Cracked marble, tarnished gold, rusted iron, obsidian glass.
- Lighting: Shafted vertical beams, volumetric fog, high contrast near the peak.
- Silhouette: Everything points upward. Spires, chains, angular machinery fused with organic curves.
- Loot Feedback: Weapons glow with rarity colors; Legendaries have unique particle effects.

### Audio
- Music: Industrial cathedral ambience, distant choral fragments, metallic percussion.
- SFX: Deep echo reverb, groaning chains, wind howling vertically.
- Feedback: Distinct "chimes" for loot rarity; unique "jam" sounds for overheating.

## Narrative Structure

The story is told through environmental decay and the player's pursuit of the "Perfect Loadout."

- Act I: Discovery. The player learns the mechanics, explores the lower Strata, and understands the threat.
- Act II: Escalation. Reality destabilizes. Enemies become more aggressive. The player must optimize their build to survive.
- Act III: Resolution. The final ascent. The player reaches the summit with their ultimate build.
- Finale: The revelation that the surface is a dead universe. The choice to rest (end save) or continue the climb (New Game+).

## Development Philosophy

### Thematic Pillars
1. The Great Omission: Reality is missing something. Loot is the only tangible memory.
2. Vertical Regret: Height equals proximity to truth. The climb is the only way forward.
3. Memory vs. Illusion: Perks are the only "truth" retained; the rest is a test of skill.
4. Beauty in Ruin: Decayed elegance, not grimdark brutality.

### Quality Control
- No Generic Mechanics: If a mechanic doesn't serve exploration, loot, or verticality, it is cut.
- Thematic Consistency: Every system must reinforce the feeling of ascending a dying world.
- Multiple Paths: Every zone must have at least 3 distinct routes (High, Mid, Low).

### Technical Standards
- Performance: Optimized for large vertical spaces and dynamic lighting.
- Extensibility: Modular code architecture for loot generation and procedural events.
- Cross-Platform: Designed for PC and Console compatibility.

## Future Roadmap (High Level)

- Phase 1: The Tight Core – Engine, Physics, Basic Movement, Loot Generation.
- Phase 2: Open-Zone & Content – 7 Strata, Enemy AI, Perk Trees, Vendors.
- Phase 3: Polish & Scale – Boss Encounters, Audio Integration, Endgame Modes, Leaderboards.

Note: This document serves as the living design bible. Specific implementation details may evolve, but the core loop and thematic pillars remain constant.
