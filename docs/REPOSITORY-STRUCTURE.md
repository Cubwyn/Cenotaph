# Repository structure

This repository is meant to be a home for a fantasy world that can grow in many directions at once.

The point isn’t to force everything into one clean, perfect project. The point is to keep the world alive and let different ideas live side by side without getting in each other’s way.

## The basic idea

Each category can hold many different projects.

A story project lives in `text/projects/`.
A game idea lives in `games/projects/`.
An art direction or character pack lives in `art/projects/`.
A rough concept or abandoned idea goes in `ideas/projects/`.

Everything still belongs to the same world, but each project keeps its own little kingdom.

## Folder layout

```text
Cenotaph/
├── shared/
│   ├── README.md
│   ├── canon/
│   ├── references/
│   └── timeline/
├── world/
│   ├── README.md
│   └── projects/
├── text/
│   ├── README.md
│   └── projects/
├── games/
│   ├── README.md
│   └── projects/
├── art/
│   ├── README.md
│   └── projects/
├── ideas/
│   ├── README.md
│   └── projects/
├── templates/
│   ├── project-readme.md
│   ├── project-brief.md
│   └── lore-entry.md
└── docs/
    ├── project-template.md
    └── REPOSITORY-STRUCTURE.md
```

## How to use it

### `shared/`
Use this for big world things that multiple projects might share.

Examples:
- core lore that feels true across the setting
- art references and inspiration
- the timeline of major events

### `world/`
This is the world itself. It’s where the setting can breathe.

This can contain:
- geography and regions
- cultures and factions
- locations and landmarks
- big historical notes

### `text/`
This is for writing projects.

Each story, journal, or prose project gets its own folder under `text/projects/`.

### `games/`
This is for game concepts, systems, prototypes, and design experiments.

Each idea should have its own folder, even if it shares the same world.

### `art/`
This is for visuals, concept work, character designs, environment studies, and mood pulls.

Every art project can stand alone.

### `ideas/`
This is for the weird, exciting, not-yet-fully-formed stuff.

Good for:
- abandoned ideas
- future experiments
- rough concepts
- side thoughts that might become something later

## The important part

There should never be a single giant folder full of unrelated scraps.

If something is a project, it gets a project folder.

If it’s a shared world fact, it belongs in the shared/world area.

That keeps the repo readable, useful, and actually enjoyable to work in.
