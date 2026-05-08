CENOTAPH: THE GREAT OMISSION
============================
Project Document

Version: 0.33
Start Date: 2026-01-29
Status: Architectural Refactoring Complete


EXECUTIVE SUMMARY
-----------------
Genre: First-Person Action RPG / Shooter
Structure: Vertical Metroidvania with modular loot systems
Tone: Surreal decay, existential grandeur, tragic beauty
Core Fantasy: Climb a dead kingdom's corpse to discover why reality forgot it.


ELEVATOR PITCH
--------------
A modular loot shooter set inside a sunken mountain made of crushed cathedrals and machine remains. 

Players ascend through layered strata of forgotten architecture while manipulating memory and causality. The higher they climb, the more reality destabilizes—until they breach the surface and discover the universe has already ended.


PROJECT STRUCTURE (0.33)
------------------------

The project has been refactored for clarity and maintainability:

src/
├── app.rs            # Application entry point & Event loop
├── main.rs           # Module orchestration
├── core/             # Engine core (State, Loader, etc.)
├── systems/          # Subsystems (Input, Physics, Render)
├── game/             # Gameplay systems (Stamina, Player)
├── data/             # World, Textures, Config
└── dev/              # Development tools (Editor)


CORE ARCHITECTURE
-----------------

1. World Structure
The Great Omission: A tectonic shaft of impossible depth.
The Pillar of Regret: A mountain formed from crushed architecture, petrified roots, and ancient machinery.
The Obsidian Spire: A black glass tower piercing the cavern ceiling into the dead world above.

2. Strata Overview (Playable Biomes)
(Retained from previous versions: The Ash-Walk, The Ward of Irons, The Hanging Slums, The Sanctuary, The Gallery of Wind, The Mirror-Crust, The Breach)


CORE GAMEPLAY SYSTEMS
---------------------

1. Combat Philosophy
Fast, responsive FPS combat with vertical mobility.

2. Modular Relic Weapons
(Schizoid, Moonchild, Sovereign parts)

3. Resonance System
The Bell-Clapper: Ringing the Great Bell unlocks previously "silenced" areas, reveals omitted objects, and changes enemy behavior in lower strata.

4. Vertical Traversal
Movement Toolkit: Combat dodge, rope/chain swinging, and chain, rock, and building climbing.


ENEMY FACTIONS
--------------
(Burdened, Silencers, Paranoiacs, Harpies)


DEVELOPMENT PRIORITIES
----------------------

Phase 1: Foundation (Complete)
- [x] Engine rendering pipeline (WGPU)
- [x] Physics integration (Rapier3D)
- [x] Basic movement and vertical traversal 
- [x] Configuration handling and generic inputs
- [x] Architectural refactoring (v0.33)
- [x] Polish player physics controller

Phase 2: Core Gameplay Mechanics (In-Progress)
- [ ] Enemy AI and basic combat loop
- [ ] Save features
- [ ] Modular weapon generation
- [ ] Bell resonance progression mechanic

Phase 3: Content Creation (Planned)
- [ ] Complete all 7 strata
- [ ] Boss encounters and enemy factions
- [ ] Audio, music, and atmosphere integration


CONCLUSION
----------

Cenotaph: The Great Omission (v0.33) has transitioned from a monolithic foundation to a modular, tree-based architecture, significantly improving code readability and maintainability. The core engine and traversal systems are stable, and the project is now well-positioned for the intensive content-creation phase.

Document Maintained By: _Cubwyn
Last Updated: May 5, 2026
