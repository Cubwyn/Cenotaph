# Cenotaph Foundation Manual Smoke Checklist

Use this after `scripts/project_check.ps1` passes and before treating the
foundation build as stable.

The goal is not balance approval. The goal is to confirm that GPU launch,
input, render, audio, physics, and foundation combat still work together.

## Preflight

- [ ] Run `powershell -ExecutionPolicy Bypass -File scripts/project_check.ps1`.
- [ ] Confirm the script reports `Project check passed.`
- [ ] If console position logs are too noisy, set
  `debug.position_log_enabled = false` in `config/tuning.toml`.

## Ash-Walk Launch

Run:

```powershell
cargo run -- play ashwalk_01
```

Pass checks:

- [ ] Window opens without panic.
- [ ] The Ash-Walk map renders; startup rejects missing or malformed base maps
  with a useful console error instead of substituting geometry.
- [ ] Mouse look updates the camera smoothly.
- [ ] `WASD` movement works and does not drift when released.
- [ ] `Space` jumps, falling returns to stable ground, and collision feels sane.
- [ ] `Shift` sprint changes speed and drains stamina on the HUD.
- [ ] `Q` dash fires once per press, consumes stamina, and respects cooldown.
- [ ] `I` does not swap relics before at least two relics are owned.
- [ ] `F5` reloads tuning, bindings, models, textures, enemy/relic definitions,
  and the current level together.
- [ ] A malformed file makes `F5` report a rejected reload while the current
  playable scene remains intact.
- [ ] HUD text is readable against bright and dark map surfaces, including the
  guide strip and event feed.
- [ ] `Escape` pauses, releases cursor, shows pause overlay, and resumes cleanly.
- [ ] Ambient audio starts, pauses, and resumes with the game.

## Foundation Test Arena

Run:

```powershell
cargo run -- foundation_test
```

Pass checks:

- [ ] Decorative, resource, Anchor, static, enemy, hurtbox, and trigger props render.
- [ ] Walking over the small resource prop logs unsecured resource collection.
- [ ] Walking into the Anchor logs activation and banks unsecured resource.
- [ ] Walking over a relic pickup adds it to owned relics; the first relic
  equips, later relics are stored without replacing the equipped relic.
- [ ] Pressing `I` after owning at least two relics cycles the equipped relic and
  the ascent HUD updates.
- [ ] Death after collecting unbanked resource logs unsecured resource loss.
- [ ] Respawn uses the active Anchor after it has been activated.
- [ ] Static props block movement as expected.
- [ ] Left mouse primary fire can hit the enemy silhouette.
- [ ] Static walls block primary fire; shooting a wall produces blocked-shot
  HUD feedback and does not damage enemies behind it.
- [ ] Clean misses produce miss feedback, while hits and kills produce distinct
  hit/kill feedback.
- [ ] Enemy health decreases and the enemy prop disappears when defeated.
- [ ] Enemy chases when inside activation range.
- [ ] Enemy attack wind-up/cooldown deals damage without rapid-fire damage spam.
- [ ] Hurtbox proximity damages the player at a readable interval.
- [ ] Player death triggers hit feedback, death audio, delayed respawn, and health
  restoration.
- [ ] Transition trigger loads the target level without panic.

## Standalone Level Editor

Run:

```powershell
cargo run -- editor
```

Pass checks:

- [ ] The terminal prints a localhost URL and the browser loads the editor.
- [ ] Opening an `/api/*` URL without the printed token returns `403`; the
  browser editor loaded from the printed URL can still list levels.
- [ ] The Levels panel lists existing `levels/*.json` files with names and prop
  counts.
- [ ] Selecting `movement_test` loads its props into the object list, Camera
  viewport, and Top/Front/Side orthographic panels.
- [ ] `4 View` shows Camera, Top X/Z, Front X/Y, and Side Z/Y panels; `Camera`
  returns to a single 3D viewport.
- [ ] `WASD`/`QE`, mouse wheel, right-drag look, and `Reset Camera` navigate the
  FPS-style viewport without losing selection or dirty state; moving the mouse
  right turns the Camera view right.
- [ ] The Keys workspace shows current bindings, can remap a command, persists
  the new binding after refresh, and Reset restores defaults.
- [ ] Create, Props, Assets, Events, Loot, Paths, Dialogue, Validate, and Keys
  workspaces switch without resizing or hiding the active viewport.
- [ ] Right-click opens the editor command menu without the browser menu; a
  right-drag still looks/pans instead of opening the menu.
- [ ] Context commands can select/place/create brush/focus/duplicate/delete and
  validate without saving unexpected files.
- [ ] Orthographic panels can pan with right/middle drag, zoom with mouse wheel,
  select props, place props, and move selected props on visible axes.
- [ ] Ctrl-click adds/removes props, Shift-click selects object-list ranges,
  Select All/Invert work, and a Select-tool drag in an orthographic panel
  marquee-selects every intersecting prop with one white primary outline and
  teal secondary outlines.
- [ ] Palette placement creates geometry, pickups, enemies, Anchors, hazards,
  and gates at ray-picked 3D ground positions.
- [ ] Place and Draw work independently in Camera, Top, Front, and Side; placing
  in both bottom viewports creates a prop, and drawing in Front/Side extrudes on
  the configured Front Z/Side X work plane instead of the Top Y plane.
- [ ] The Draw tool drag-creates floor, wall, block, slope, cylinder, stair, and
  terrain brush geometry using the configured extrusion, thickness, work plane,
  direction, segment, step, terrain grid, relief, and seed values.
- [ ] Slope direction, cylinder side count, and stair step/direction controls
  create visible Mesh-collider brushes that pass validation, save, reload, and
  hot reload without degenerate-face or missing-asset errors.
- [ ] A terrain brush is a closed mesh, exposes Raise/Lower/Smooth/Flatten and
  seed regeneration controls, keeps its raw 162+ vertex mesh collapsed by
  default, and still passes validation after sculpting.
- [ ] Prefabs lists Basic Room Shell; placing it from Top, Front, and Side adds
  five selected props with unique IDs, validates successfully, and one Undo
  removes the whole group.
- [ ] Create from Selection writes a reusable relative prefab without dirtying
  the current level; overwriting or deleting it creates an ignored backup, and
  traversal-like prefab IDs are rejected by the server.
- [ ] Select and Move tools can ray-pick props and move them on the X/Z plane.
- [ ] Multi-selected props move together without changing relative spacing;
  XYZ/X/Y/Z constraints, Snap, grid size, pivot input, nudges, yaw, scale, and
  reset controls affect the intended axes only.
- [ ] One drag or focused numeric edit requires one Undo, Redo reapplies it,
  and undoing back to the loaded snapshot changes the level badge from Unsaved
  to Saved.
- [ ] After an edit, the level badge advances from Draft pending to Draft saved
  locally; reopening the level offers recovery, and one Undo from the recovered
  draft restores the exact on-disk level and clears the local draft.
- [ ] Default `F`, `Ctrl+D`, `Ctrl+C`, and `Ctrl+V` focus, duplicate, copy, and
  paste selected props.
- [ ] The Asset Browser lists model, texture, material, audio, dialogue/data,
  level, and config files; runtime model clicks become placeable templates, and
  source-only file clicks stage `asset_imports` without saving until `Save`.
- [ ] The Inspector edits level metadata, prop asset, transform, collider,
  gameplay fields, and advanced graph JSON.
- [ ] Validate reports `OK` for good data and shows issue rows for broken data.
- [ ] Save writes the real `levels/<level>.json` only after validation passes.
- [ ] Saving over an existing level creates a backup in
  `levels/.editor_backups/`.
- [ ] Running the game on the same level hot reloads clean saved changes.

## Runtime Quick Adjuster

Run:

```powershell
cargo run -- movement_test
```

Pass checks:

- [ ] `Tab` toggles editor mode and shows the editor HUD panel.
- [ ] While editor mode is active, pickups, hurtboxes, enemy AI, combat, and
  transition triggers do not mutate the level during editing.
- [ ] Mouse look and movement can still be used to navigate the level.
- [ ] The editor cursor snaps to the crosshair surface or placement distance.
- [ ] `G` cycles geometry, item, enemy, and entity placement modes.
- [ ] Left/right arrows cycle templates within the active mode.
- [ ] `Enter` places geometry brushes, resource/relic pickups, enemies, Anchors,
  hazards, and transition gates into the live level.
- [ ] Up/down arrows select existing props and `Delete` removes the selected prop.
- [ ] `V` validates the current level in-editor and updates the editor HUD check
  row with `OK`, `CHECK NEEDED`, or an issue count.
- [ ] `P` writes the current level JSON after validation passes; invalid data is
  blocked with HUD feedback and detailed console errors.
- [ ] `R` reloads the current level from disk; dirty levels require pressing `R`
  twice before unsaved edits are discarded.
- [ ] External edits to `levels/<current>.json` hot reload while the editor is
  clean and warn instead of overwriting when dirty.
- [ ] `cargo run -- validate` catches broken editor-authored graph data:
  duplicate IDs, missing model assets, bad loot table relic IDs, missing
  paths/events/dialogues, invalid event actions, and malformed path waypoints.
- [ ] A level with an `OnEnter` or proximity event can grant resource, spawn
  loot from a loot table, print dialogue, set a level-local flag, or queue a
  level transition during play mode.
- [ ] Once-only level events do not refire after saving and continuing, and
  flag-gated events wait until their required level flag has been set.
- [ ] A prop/enemy with a valid `path_id` follows its authored path in play mode
  and stops/loops according to the path definition.

Useful advanced-editor test level:

```powershell
cargo run -- editor_systems_test
```

## Save / Continue

After collecting resource and at least one relic, exit and run:

```powershell
cargo run -- continue
```

Pass checks:

- [ ] The saved level loads without silent level substitution.
- [ ] Banked resource, active cycle, owned relic count, and equipped relic are
  restored in the HUD/console output.
- [ ] Fired once-events and level-local event flags are restored from the save
  file instead of replaying editor-authored rewards on continue.
- [ ] Runtime autosaves remain local developer state and do not appear as
  trackable project content.
- [ ] `cargo doctor` reports the primary save and its recovery state accurately.

## Failure Notes

For every failed check, record:

- level command used
- observed behavior
- console output around the failure
- GPU/driver details if launch or rendering failed
- whether `cargo doctor` still passes
