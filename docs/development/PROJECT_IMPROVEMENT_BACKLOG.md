# Project-Wide Improvement Backlog

This is the ordered backlog for improvements that affect the game runtime,
content pipeline, testing, and everyday development. Standalone editor work
continues in `docs/editor/LEVEL_EDITOR_BACKLOG.md`.

Status key: `[x]` complete, `[ ]` pending. Finish and verify one item before
marking it complete.

## Reliability Baseline

- [x] Strict command parsing, safe level IDs, explicit play syntax, and useful
  help/level-list errors.
- [x] Project doctor for layout, content, save health, source/runtime hygiene,
  editor backups, and pending write sidecars.
- [x] Strict startup for tuning, bindings, enemy/relic registries, level JSON,
  and base-map geometry.
- [x] Transactional F5 reload across config, bindings, registries, models,
  textures, and the current level.
- [x] Shared staged writes with cross-process locking and interrupted-write
  recovery for levels, prefabs, and saves.
- [x] Validated autosaves with a rolling last-known-good backup.
- [x] Full project check covering format, all-target Clippy, tests, and doctor.
- [ ] Version every long-lived authored data contract and add explicit migration
  functions instead of ad hoc compatibility defaults.
- [ ] Add save profiles/slots, metadata previews, manual backup restore, and
  forward-only save migrations.
- [ ] Add a crash report containing build ID, active level, recent structured
  events, graphics adapter, and content hashes.

## Fast Iteration

- [ ] Add an in-game developer console with command completion, history,
  discoverable help, and permission-gated mutating commands.
- [ ] Build an asset dependency graph so reload can update only changed models,
  textures, definitions, and affected draw groups.
- [ ] Add shader hot reload with validation and last-known-good pipeline fallback.
- [ ] Add a runtime diagnostics overlay for frame time, physics time, draw calls,
  triangles, entities, memory estimates, and reload status.
- [ ] Add deterministic input recording/replay for reproducing movement, combat,
  transition, and editor bugs.
- [ ] Add developer teleport, level bookmark, free-camera, time-scale, and
  invulnerability commands through the shared console.
- [ ] Add a content packaging command that emits a clean runtime bundle and
  rejects source-only or unreferenced production files.

## Automated Confidence

- [ ] Add headless gameplay scenarios for spawn, movement, combat, pickups,
  death, respawn, Anchor banking, events, and level transitions.
- [ ] Add deterministic multi-level ascent simulations with seeded loot and cycle
  modifiers.
- [ ] Add snapshot tests for every authored JSON/TOML schema and migration.
- [ ] Add save interruption/fault-injection tests around every persistence phase.
- [ ] Add render smoke captures for representative levels and compare nonblank
  output, framing, and key HUD regions.
- [ ] Add performance budgets and regression checks for startup, reload, frame
  time, collider count, and asset memory.
- [ ] Add CI profiles for fast checks, full content validation, render smoke, and
  packaged-build smoke tests.

## Runtime Architecture

- [ ] Split the large engine state/update modules by ownership while preserving a
  single explicit frame schedule.
- [ ] Introduce stable entity handles so runtime state no longer depends on prop
  vector indices.
- [ ] Separate authored level data from mutable runtime entity state.
- [ ] Add a queued event bus for gameplay, presentation, audio, save, and debug
  events with bounded history.
- [ ] Add level streaming/loading states with progress, cancellation, and clean
  failure UI instead of blocking the frame loop.
- [ ] Add runtime asset reference counting and unload resources no active level
  needs.
- [ ] Add structured logging with levels, categories, timestamps, and rotating
  local logs.

## World And Gameplay Tooling

- [ ] Add navmesh generation/import, validation, path preview, blocked-path
  diagnostics, and runtime navmesh reload.
- [ ] Add encounter definitions, spawn groups, waves, budgets, and encounter
  simulation reports.
- [ ] Add loot-table probability analysis, unreachable-item detection, and seeded
  roll previews.
- [ ] Add dialogue localization IDs, condition/action graphs, speaker validation,
  and conversation-state tests.
- [ ] Add event graph tracing, breakpoint-style debug pauses, and fired-action
  history.
- [ ] Add run-contract and Cycle Director authoring data with deterministic
  preview/simulation tools.
- [ ] Add relic affix generation, compatibility rules, balance summaries, and
  build comparison tooling.
- [ ] Add enemy behavior trees/state machines with visual debug state, perception
  probes, and deterministic AI tests.

## Player-Facing Completeness

- [ ] Add settings UI for controls, mouse, audio, graphics, accessibility, and
  safe reset-to-default behavior.
- [ ] Add save-slot UI with corruption/recovery messaging and profile metadata.
- [ ] Add loading/error screens that explain rejected content or device failures
  without requiring a terminal.
- [ ] Add complete inventory, item comparison, Anchor/Sanctuary, run summary, and
  Cycle Director interfaces.
- [ ] Add accessibility passes for text scale, contrast, reduced motion, camera
  shake, hold/toggle inputs, captions, and color-independent feedback.

## Release And Maintenance

- [ ] Embed semantic build/version information and authored schema versions in
  packaged builds and crash reports.
- [ ] Add deterministic release packaging, checksums, and a generated content
  manifest.
- [ ] Add configuration tiers for development, playtest, profiling, and release.
- [ ] Add compatibility checks for GPU limits, audio availability, writable save
  paths, and unsupported platforms before gameplay starts.
- [ ] Add automated backup retention policies for editor history, saves, logs,
  screenshots, and crash reports.
