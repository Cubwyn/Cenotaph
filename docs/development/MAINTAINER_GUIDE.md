# Cenotaph Maintainer Guide

This is the short version of how to work on Cenotaph when you do not want to
hold the whole codebase in your head. The project is intentionally arranged so
that most expansion happens in data files and only new behavior requires Rust.

## Start here

Run:

```powershell
cargo content
```

This prints the current content areas and the safe development loop. It is a
map, not a validator. When something is broken, use:

```powershell
cargo validate-content
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/project_check.ps1
```

The first command catches content references and schema mistakes. The second
checks formatting, Clippy, tests, and project health. For a runtime change,
also use `docs/development/FOUNDATION_SMOKE_CHECKLIST.md`.

## Where things belong

| If you want to change... | Start here | Usually avoid changing... |
| --- | --- | --- |
| A level layout, prop, event, dialogue, route, or transition | `levels/*.json` | `src/core/engine/update.rs` |
| A reusable group of props | `prefabs/*.json` | Copying the same group into many levels |
| Enemy numbers or an existing enemy role | `data/enemies/*.toml`, `config/tuning.toml` | Rust enemy logic |
| Relic stats, name, flavor, or pickup model | `data/relics/*.toml` | Hard-coded relic IDs in runtime code |
| Controls or player-facing tuning | `config/bindings.toml`, `config/tuning.toml` | Scattered numeric constants |
| A new gameplay rule | `src/game/` first | Putting game rules in rendering code |
| Level loading, reload, save, or content validation | `src/core/` | Duplicating a second loading path |
| HUD, models, particles, input, physics, or audio plumbing | `src/systems/` | Reaching into unrelated gameplay state |
| A check, report, or developer workflow | `src/developer/`, `scripts/`, `docs/development/` | One-off commands nobody can repeat |

The practical rule is: change the smallest owner that already understands the
thing. If the change needs a new owner, add a small module with a narrow API
before growing an existing large file.

## Safe content recipes

### Add an enemy

1. Copy the closest file in `data/enemies/`.
2. Give it a unique normalized `id`, a clear `display_name`, and an existing
   `behavior_tag`.
3. Reuse a deliberately simple model under `assets/` unless visual direction
   has been supplied. Do not infer anatomy or ornament from the name.
4. Add the enemy to a test level only after the definition validates.
5. Run `cargo validate-content`, then `cargo content` and the project check.

The existing behavior tags are the extension seam. A new behavior needs code;
a new enemy using an existing behavior should remain data-only.

### Add a relic

1. Copy the closest file in `data/relics/`.
2. Change the ID, authored display name, family, rarity, multipliers, and
   flavor text.
3. Reuse an existing pickup model when possible. `pickup_asset` is relative to
   `assets/` and must not introduce a hard-coded Rust asset match.
4. Add a stable authored loot source if the relic is granted by an event or
   encounter.
5. Run validation before launching the game.

### Add a level or event

1. Copy the smallest similar level, keeping `version` and schema shape intact.
2. Use stable IDs for props that events, paths, loot, dialogue, or saves need.
3. Keep authored content in the level; keep reusable groups in `prefabs/`.
4. Reference existing models and materials before adding assets.
5. Validate, then launch with `cargo run -- play <level-id>`.

### Add a runtime feature

1. Write down the player decision and the failure/counterplay first.
2. Identify whether the rule belongs to `src/game`, `src/core`, or a system
   under `src/systems`.
3. Keep authored values in data/config and pass typed or validated values into
   runtime code.
4. Add a focused unit test for the rule and a content/runtime test if it crosses
   a level boundary.
5. Preserve the explicit frame order in `src/app.rs` and the transactional
   `prepare_level`/reload boundary in the engine.

If the feature requires a new string ID, add validation at the same time. A
string reference without a validator is a future typo disguised as content.

## The five-minute decision test

Before editing, answer these questions in a note or commit message:

- What is the smallest file that owns this change?
- Is this content, tuning, or genuinely new behavior?
- What existing asset/behavior/schema can be reused?
- What command will prove the change is valid?
- What manual smoke step proves it works inside an ascent?

If the answer is unclear, run `cargo content`, read the relevant section of
`docs/design/CONTENT_GUIDE.md`, and inspect one neighboring working example.

## Things to keep stable

- `EngineState::prepare_level` is the strict runtime preparation boundary.
- `EngineState::reload_runtime_content` replaces live content transactionally.
- `src/core/persistence.rs` is the shared staged-write path.
- `cargo validate-content` is the non-window content gate.
- `scripts/project_check.ps1` is the canonical code-and-health gate.
- Runtime placeholders stay honest single primitives; semantic audio hooks stay
  silent until authored recordings exist.

When a change touches one of these, add or update a test and update the relevant
development document in the same change.

## When the project grows

Prefer one small content pack over a broad system expansion. A pack should be
easy to identify in version control, validate independently, and remove without
leaving hidden references. The templates in
`docs/design/CONTENT_GUIDE.md` define the design questions; the data files and
validation rules define the executable contract.
