# CENOTAPH: THE GREAT OMISSION
## Project Document

**Version:** 0.333

**Start Date:** 2026-01-29  

**Status:** Open-Zone & Loot Loop Definition

---

## Executive Summary

* **Genre:** Vertical Action RPG / Open-Zone Loot Shooter
* **Structure:** 7 Massive, Non-Linear Strata with branching paths, side quests, and hidden zones.
* **Tone:** Surreal decay, tragic beauty, chaotic exploration.
* **Core Fantasy:** Explore a vast, dying world, hunt for legendary weapons, and climb the mountain in your own unique way.

## Elevator Pitch

A modular loot shooter set inside a sunken mountain made of crushed cathedrals and machine remains. 

*Cenotaph* features 7 massive, open Strata filled with branching paths, hidden caves, and dynamic events. Players explore freely, choosing their own route to the summit. Every enemy drops randomized, fully-formed weapons with unique stats and effects. There are multiple ways to solve every encounter: stealth, brute force, or environmental manipulation. The mountain resets on death, but permanent upgrades and the sheer volume of loot ensure infinite replayability.

---

## Core Architecture

### 1. World Structure

* **The Great Omission:** A tectonic shaft of impossible depth. Walls are geological layers fused with compressed cities, bone deposits, and cathedral remains.
* **The Pillar of Regret:** A mountain formed from crushed architecture, petrified roots, and ancient machinery. This is the main play space.
* **The Obsidian Spire:** A black glass tower piercing the cavern ceiling into the dead world above.
* **Anchor Chains:** Colossal chains bolted into the cavern ceiling stabilizing the mountain against violent updrafts.

### 2. Strata Overview (Open Zones)

* **The Ash-Walk**
  * *Theme:* Collapse
  * *Visuals:* Wagon graveyards, fossilized tree roots, dust storms
  * *Palette:* Sepia, pale red, smoke gray
  * *Traversal:* Wide open spaces, multiple vertical routes, hidden wagon caches.
  * *Gameplay:* Multiple entry points to the next zone; optional "Wagon Graveyard" side quest.
* **The Ward of Irons**
  * *Theme:* Industrial insanity
  * *Visuals:* Bleeding steam pipes, iron scaffolds, suspended torture machinery
  * *Palette:* Rust red, oil black, tarnished brass
  * *Traversal:* Maze-like factory floors, elevator shafts, rooftop shortcuts.
  * *Gameplay:* Dynamic events (steam vent cycles); optional "Power Core" side objective.
* **The Hanging Slums**
  * *Theme:* Precarious life
  * *Visuals:* Houses pinned to cliff walls, floating debris, grand pianos tangled in wires
  * *Palette:* Desaturated teal, faded wood tones
  * *Traversal:* Rooftop parkour, chain bridges, hidden basement tunnels.
  * *Gameplay:* Stealth opportunities; optional "Rescue" side quests for unique loot.
* **The Sanctuary**
  * *Theme:* Broken divinity
  * *Visuals:* Gothic cathedral suspended in chains
  * *Palette:* Gold leaf, ivory stone, dim violet light
  * *Traversal:* Central Hub (Safe Zone).
  * *Gameplay:* Vendor, Quest Board, Perk upgrades, Fast Travel to Strata entrances.
* **The Gallery of Wind**
  * *Theme:* Breath of the abyss
  * *Visuals:* Massive carved trumpets and wind tunnels
  * *Palette:* Pale limestone, wind-swept silver
  * *Traversal:* Aerial currents, floating platforms, deep vertical drops.
  * *Gameplay:* High-speed traversal challenges; optional "Wind Singer" side quest.
* **The Mirror-Crust**
  * *Theme:* Reality distortion
  * *Visuals:* Glass trees, upward-flowing water, reflective terrain
  * *Palette:* Silver-blue, glass white
  * *Traversal:* Slippery slopes, mirror portals, hidden reflection rooms.
  * *Gameplay:* Puzzle-solving for shortcuts; optional "Shattered Memory" side quest.
* **The Breach**
  * *Theme:* Geometric collapse
  * *Visuals:* Obsidian floors, fracturing space
  * *Palette:* Black, stark white, color draining
  * *Traversal:* Final ascent with multiple branching paths to the peak.
  * *Gameplay:* Elite enemy camps; optional "Boss Rush" challenge.

---

## Core Gameplay Systems

### 1. Combat Philosophy

**Fast, responsive FPS combat with vertical mobility and multiple approaches.**
* **Open Exploration:** Players choose their path through each Stratum.
* **Multiple Approaches:**
  * *Brute Force:* Overwhelm enemies with heavy firepower.
  * *Stealth:* Use shadows and silence to bypass groups.
  * *Environmental:* Lure enemies into traps or use verticality to snipe.
* **Randomized Loot:** Every enemy and chest drops a **fully formed weapon** with random stats, elements, and effects. No assembly required.
* **Verticality as Tactical Advantage:** Use recoil to boost upward, drop-kicks for damage, and wall-runs for evasion.

### 2. Modular Relic Weapons (Randomized Drops)

* **Weapon Types:** Schizoid (Chaotic), Moonchild (Homing), Sovereign (Heavy), plus hybrids.
* **Randomization System:**
  * **Prefixes/Suffixes:** "The Weeping," "The Regretful," "The Burning," "The Void."
  * **Stats:** Fire Rate, Damage, Spread, Heat Gen, Reload Speed, Element (Fire, Ice, Shock, Void).
  * **Special Effects:** "Hitscan," "Bouncing Bullets," "Explosive Rounds," "Life Steal."
* **Loot Philosophy:** Every drop is unique. Finding a "Legendary" weapon with a crazy effect is the core reward loop.

### 3. Progression System

* **The Infinite Loop:** Upon death, the mountain collapses and regenerates.
* **Persistent Upgrades:** Players retain **Scrap** (currency) and **Memory Shards** to unlock permanent **Perks** at the Sanctuary.
* **Perk Slots:**
  * *Movement:* Double Jump, Wall-Run Speed, Dash Cooldown.
  * *Combat:* Loot Magnet, Crit Chance, Heat Capacity.
  * *Survival:* Max HP, Damage Resistance, Revive Chance.
* **Quest System:** Each Stratum has a Main Objective (Reach the top) and 2-3 Side Quests (Find a specific item, clear a camp, rescue an NPC) that reward unique loot or lore.
* **Difficulty Scaling:** Each loop increases the "Madness Level," spawning harder enemies and rarer loot.

### 4. Vertical Traversal

* **Movement Toolkit:** Combat dodge, rope/chain swinging, wall-running, and climbing.
* **Recoil Dash:** Shooting pushes the player backward; aiming down allows upward propulsion (rocket-jump style).
* **Drop-Kick:** Falling onto enemies deals massive damage and bounces the player up.
* **Design Principle:** The world must reward height constantly. Multiple paths (high, mid, low) exist for every section.

---

## Enemy Factions

* **The Burdened:** Stone-carrying husks. Slow but high impact. *(Design: Physical weight and momentum)*
* **The Silencers:** Tall figures with mouths sewn shut in gold wire. Suppress player abilities temporarily. *(Design: Anti-magic, silence mechanics)*
* **The Paranoiacs:** Cage-headed screamers. Alert entire zones if not eliminated quickly. *(Design: Area denial, escalation)*
* **The Harpies:** Marble angel statues with decayed wings. Agile threats. *(Design: Aerial combat, vertical threats)*
* **Elite Variants:** Randomly generated enemies with unique modifiers (e.g., "Fire Burdened," "Fast Silencer") found in higher Strata.
* **Boss Philosophy:** Each boss represents a failed memory of the kingdom, guarding a major loot cache or quest item.

---

## Audio Direction

### Music
* Industrial cathedral ambience
* Distant choral fragments
* Metallic percussion
* Dynamic intensity that scales with "Madness Level"

### Sound Design
* Deep echo reverb
* Chains groaning under stress
* Wind howling vertically
* Distinct "jam" sound for overheating
* **Loot Feedback:** Unique chimes for Common, Rare, Epic, and Legendary drops.

---

## Narrative Structure

### Act I - Survival
* **Belief:** There is salvation above.
* **Player State:** Hope, determination, curiosity
* **Mechanic:** Exploring the first Strata, learning the loot system.

### Act II - Distortion
* **Reality:** Destabilizes. NPC accounts contradict.
* **Player State:** Confusion, paranoia, doubt
* **Mechanic:** Side quests reveal conflicting lore; Madness Level increases.

### Act III - The Infinite
* **Environment:** Color drains. Systems begin failing.
* **Player State:** Dread, inevitability
* **Mechanic:** The mountain regenerates endlessly. The goal is to find the "Perfect Loadout" and survive the highest loop.

### Finale - Epitaph (Optional)
* **Setting:** Surface is an infinite gray ash desert.
* **Revelation:** There is no kingdom to save. The mountain was the last vibrant remnant in a dead universe.
* **Choice:** The player chooses to continue the climb (New Loop) or rest (End Save).

---

## Visual Identity

### Textures
Cracked marble, tarnished gold, rusted iron, petrified wood, obsidian glass.

### Lighting
Shafted vertical light beams, soft volumetric fog, high contrast near peak. Light quality degrades with height.

### Silhouette Language
Spires, chains, vertical thrust. Angular machinery fused with organic curves. Everything points upward.

### Loot Feedback
Weapons glow with rarity colors (White, Green, Blue, Purple, Orange) upon pickup. Legendary items have unique visual effects (e.g., trailing fire, static).

---

## Thematic Pillars

### The Great Omission
**Reality is missing something.** Entire civilizations are erased—not destroyed, but forgotten. Absence is the central antagonist.
* **Gameplay Expression:** The mountain regenerates because reality cannot remember its end. Loot is the only tangible memory.

### Vertical Regret
**The world is a wound that collapsed inward.** Height equals proximity to truth.
* **Gameplay Expression:** Vertical traversal is primary. The climb is the only way forward.

### Memory vs. Illusion
**The mountain may be the last "living" thing.** The surface may be sterile oblivion.
* **Gameplay Expression:** Permanent upgrades (Perks) are the only "truth" retained between loops. The rest is an illusion.

### Beauty in Ruin
**This is not grimdark brutality.** It is decayed elegance.
* **Art Direction:** Faded gold leaf, marble veined with rust, pale greens, ash whites, bruised purples, organic roots fused with industrial machinery.

---

## Design Constraints

### Do Not Dilute
Vertical scale, surreal architecture, oppressing feel, bleak but elegant tone.

### Quality Control
If something feels generic, remove it. If a mechanic doesn't serve the exploration or the loot, cut it.

---

## Development Priorities

### Phase 1: The Tight Core (Current)
* [x] Engine rendering pipeline (WGPU)
* [x] Physics integration (Rapier3D)
* [x] Basic movement and vertical traversal (Wall-run, Chain-swing, Recoil Dash)
* [x] Configuration handling and generic inputs
* [in progress] Randomized Loot Drop System (Fully formed weapons)
* [ ] Polish player physics controller
* [ ] Implement Heat System and Overheat logic

### Phase 2: Open-Zone & Content (Planned)
* [ ] Build 7 Massive Strata with branching paths and side quests
* [ ] Enemy AI and dynamic event system
* [ ] Perk System and Sanctuary Hub
* [ ] Quest Board and NPC interaction

### Phase 3: Polish & Scale (Planned)
* [ ] Complete all 7 strata and level variations
* [ ] Boss encounters and Elite variants
* [ ] Audio, music, and atmosphere integration
* [ ] Leaderboards and Challenge Modes

---

## Guidelines

### Creative Direction
* Maintain thematic consistency across all systems.
* Prioritize verticality in level design.
* Ensure every mechanic serves the core loop (Explore -> Loot -> Fight -> Die -> Repeat).
* Preserve the tone of tragic beauty.
* **Multiple Paths:** Every zone must have at least 3 distinct routes (High, Mid, Low) to encourage replayability.

### Technical Standards
* Performance optimization for large vertical spaces.
* Modular, extensible code architecture for loot generation and procedural events.
* Comprehensive asset pipeline.
* Cross-platform compatibility.

### Quality Assurance
* Regular playtesting for vertical traversal feel.
* Balance testing for loot drop rates and difficulty scaling.
* Narrative coherence verification.
* Audio-visual synchronization.

---

## Conclusion

**Cenotaph: The Great Omission** aims to represent a unique fusion of vertical exploration, modular progression, and infinite replayability. 

The foundation is solid, with a complete engine and first level implementation providing a strong base for further development. The project's strength lies in its cohesive thematic integration—every system, from combat to traversal to the infinite loop, serves the core concept of ascending through a forgotten reality. Maintaining this thematic purity while refining the "Recoil Dash," "Loot," and "Open-Zone" feel will be crucial as development progresses.

With the technical foundation established and creative vision clearly defined, the project is ready to move into the next phase of content creation and system implementation.

**Document Maintained By:** _Cubwyn  
**Last Updated:** May 04, 2026
