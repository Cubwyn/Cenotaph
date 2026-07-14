# Level Editor Backlog

This is the working checklist for turning the standalone level editor into the
main authoring tool for Cenotaph. The in-game editor remains a quick runtime
adjuster for smoke testing and hot-reload checks.

The rule: every feature must land through the whole project before it is
considered done: data schema, validation, runtime behavior where applicable,
editor UI, save/hot reload, tests, and docs.

## Current Slice

- [x] In-game editor toggle, cursor, grid snap, placement distance, save, reload,
  hot reload, dirty-state protection, and safe deletion.
- [x] User-friendly compact editor HUD with saved/unsaved state, validation
  status, core controls, fitted message text, in-editor validation, and
  validation-blocked saves.
- [x] Standalone localhost level editor launched by `cargo run -- editor`, with
  project level list, Hammer-style Camera/Top/Front/Side view panels, 3D WebGL
  viewport, flying camera, ray-picked and orthographic placement, object list,
  inspector editing, advanced graph JSON editing, Rust-backed validation, and
  direct save-to-level-file workflow.
- [x] Draw-geometry workflow for floor, wall, and block brush creation from the
  standalone Camera, Top, Front, and Side views with explicit work planes.
- [x] Project asset browser and catalog for runtime-supported models plus
  source-only model, texture, material, audio, dialogue/data, level, and config
  files.
- [x] FPS-style editor camera look, configurable keybindings, right-click
  context command menu, focus selected, duplicate, copy, and paste.
- [x] Geometry, item, enemy, and entity template placement.
- [x] Solid-world shot obstruction, readable HUD text, blocked/miss/hit feedback,
  and physics update ordering fixes.
- [x] Expanded level authoring schema for stable prop IDs, asset imports, loot
  tables, paths, events, and dialogue blocks.
- [x] Project validation for authored graph references, duplicate IDs, model
  asset references, loot table relic references, trigger/action requirements,
  path shape, dialogue content, and save round-tripping.
- [x] Runtime level events for proximity/on-enter triggers, resource grants,
  deterministic weighted loot table rolls, dialogue logging, level-transition
  queuing, level-local event flags, and save-restored once-event state.
- [x] Runtime path following for idle enemies and non-enemy path-bound props.
- [x] Dedicated Create, Props, Assets, Events, Loot, Paths, Dialogue,
  Validation, and Keys workspaces instead of one long stacked inspector.
- [x] Multi-prop selection through Ctrl-click, Shift ranges, Select All,
  inversion, and orthographic marquee selection, with primary/secondary
  outlines and group focus/copy/paste/duplicate/delete behavior.
- [x] Transactional undo/redo that coalesces drags and focused field edits,
  tracks the saved snapshot, and clears the unsaved state when undo returns to
  the on-disk level.
- [x] Group transform workflow with axis constraints, snap toggle, configurable
  grid, numeric single-prop transforms, group pivot editing, six-axis nudging,
  yaw/scale operations, and reset controls.
- [x] Runtime-backed custom brush presets for slopes, configurable cylinders,
  and directional stairs, including conversion from existing props, mesh
  collider use, visualization, Rust validation, and save/hot-reload support.
- [x] Closed terrain heightfield brushes with configurable grid, relief, and
  seed; selected-terrain raise/lower, smoothing, flattening, regeneration,
  runtime rendering, mesh collision, validation, and save/hot-reload metadata.
- [x] Browser-local autosave drafts with reload recovery, disk-change warnings,
  exact on-disk Undo fallback, successful-save cleanup, and explicit-discard
  cleanup.
- [x] Versioned reusable prop prefabs with create-from-selection, bottom-center
  pivots, filtering, secure validated project files, backup-backed overwrite
  and deletion, collision-safe IDs, all-view placement, and one-step Undo.

## Next Slices

- [x] Editor UI: dedicated inspector tabs for Props, Assets, Events, Loot,
  Paths, Dialogue, and Validation instead of the current stacked panels.
- [x] Transform editing: move, rotate, scale, numeric values, axis locks, grid
  size control, snap toggles, and reset controls.
- [x] Selection tools: marquee/nearest selection, selected-object outline,
  group selection, and undo/redo.
- [ ] Asset import workflow: file copy/import from external paths, assign default
  scale/collider/tags, preview models/textures/audio/dialogue, and convert
  source assets into runtime-ready assets.
- [ ] Advanced geometry workflow: brush templates, collider visualization, ramp
  defaults, moving platforms, climbable geometry, and freeform mesh/CSG
  exploration.
- [ ] Event editor: create trigger volumes, choose trigger type, attach actions,
  test-fire actions, inspect fired flags, and visualize event bounds.
- [ ] Loot table editor: add weighted entries, rolls, quantities, resource drops,
  relic drops, simulation/preview, and enemy/container assignment.
- [ ] Enemy path editor: waypoint placement, loop toggle, speed multiplier, patrol
  preview, enemy assignment, and aggro-vs-patrol visualization.
- [ ] NPC/dialogue path editor: NPC placement as path-bound props, dialogue
  assignment, line editing, speaker display, interact trigger wiring, and path
  preview.
- [ ] Encounter tools: groups, waves, spawn points, patrol assignment, elite
  variants, and encounter validation.
- [ ] Navigation tools: nav markers, jump/drop links, blocked-space checks, and
  spawn safety checks.
- [ ] Lighting/audio tools: place lights, ambient audio zones, one-shot sounds,
  light intensity/color editing, and preview toggles.
- [ ] Gameplay volumes: hurtboxes, safe rooms, Anchor zones, banking zones,
  transition gates, resource zones, and scripted route blockers.
- [ ] Detailed validation UI: list issues by object, select offending object,
  jump camera to the problem, and offer guided fixes for common errors.
- [ ] Authoring workflow polish: recovery files outside browser storage,
  save-as level, commit-friendly JSON ordering, and diff-conscious writes.
- [ ] Runtime test harness: headless checks for event firing, loot spawning,
  path following, transition triggers, and level reload consistency.
- [ ] Visual QA: editor HUD readability at common resolutions, marker overlap
  checks, selected object visibility, and dev-friendly feedback for failures.

## Definition Of Done

A backlog item is not done until:

- [ ] The level file can represent it.
- [ ] Broken data is caught by `cargo run -- validate`.
- [ ] The editor can create or modify it.
- [ ] Runtime either uses it or explicitly marks it authoring-only.
- [ ] Save, reload, and hot reload preserve it.
- [ ] Tests cover the high-risk behavior.
- [ ] The smoke checklist documents how to verify it manually.
