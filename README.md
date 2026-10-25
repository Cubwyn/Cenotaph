# Cenotaph

Cenotaph is a long-term personal creative project and a central repository for the ideas, experiments, research, tooling, and documentation that support it.

This repository is organized around several distinct but related areas rather than a single monolithic project. The goal is to keep the main game, prototypes, development tooling, creative archive, visual material, and technical learning separate enough to remain manageable while still clearly tied to the same broader creative effort.

## Active repository structure

```text
Cenotaph/
├── README.md
├── game/
│   ├── README.md
│   ├── design/
│   ├── prototypes/
│   ├── notes/
│   └── archive/
├── toolchain/
│   ├── README.md
│   ├── engine/
│   ├── editor/
│   └── experiments/
├── docs/
│   ├── README.md
│   ├── lore/
│   ├── narrative/
│   ├── references/
│   ├── decisions/
│   └── history/
├── art/
│   ├── README.md
│   ├── concept/
│   ├── references/
│   └── assets/
├── learning/
│   ├── README.md
│   ├── rust/
│   └── experiments/
├── .gitignore
├── LICENSE
└── legacy/
    └── README.md
```

## Purpose of each area

- `game/` — the main Cenotaph game and all work directly tied to its creative direction.
- `toolchain/` — Rust tooling, editor infrastructure, and engine-related experiments.
- `docs/` — lore, narrative, references, design decisions, and history.
- `art/` — visual references, concept work, and asset organization.
- `learning/` — Rust and technical learning material kept separate from production work.
- `legacy/` — intentionally holds older directory layouts that should not be treated as active structure unless explicitly reopened.

## Design principles

- Keep major project categories distinct.
- Keep separate prototypes and experiments from the main game unless they are intentionally merged.
- Preserve uncertainty and version history instead of pretending every idea is final.
- Keep tools, writing, art, and learning work in their own lanes.
- Do not silently promote exploratory work into canon.

## Current status

This is a minimal, maintainable structure. It does not add lore, game mechanics, or creative decisions beyond the repository organization itself.

The repository is intentionally structured for later growth without prematurely locking the project into a final architecture.
