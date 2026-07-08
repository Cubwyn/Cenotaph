# Cenotaph Model Reset Plan

## Purpose

This is the scratch rebuild plan for models and placeholder assets.

The current assets can remain as scaffolding while the new model set is built.
The reset should be incremental: replace one category at a time, validate, then
move to the next.

## Phase 0: Freeze The Current Scaffolding

- Keep `assets/Cube.obj`, current test maps, and current enemy silhouettes for
  compatibility.
- Do not delete or rename referenced assets during the reset.
- Treat new asset paths as the future-stable IDs.

## Phase 1: Build The Blockout Kit

Create:

- `assets/props/anchor_placeholder.obj`
- `assets/props/resource_shard.obj`
- `assets/props/hazard_ash_spike.obj`
- `assets/level_kit/route_platform.obj`
- `assets/level_kit/route_ramp.obj`
- `assets/level_kit/route_gate.obj`

Acceptance:

- assets load through validation
- assets can replace cube props in `foundation_test`
- collider sizes remain obvious from the silhouette

## Phase 2: Replace Foundation Test Props

Update `levels/foundation_test.json` so the arena no longer depends on cube
visuals for gameplay meaning.

Replace:

- resource cube with `props/resource_shard.obj`
- Anchor cube with `props/anchor_placeholder.obj`
- hurtbox cube with `props/hazard_ash_spike.obj`
- transition cube with `level_kit/route_gate.obj`

Keep one decorative cube only if needed for loader sanity testing.

## Phase 3: Build The First Route Choice

Use the blockout kit to make two readable paths:

- safe route: slower, fewer hazards
- dangerous route: shorter, includes a hurtbox hazard and resource reward

Acceptance:

- player can identify the route split
- both routes reconnect near the Anchor
- resource banking has a reason to exist

## Phase 4: Enemy Production Pass

Once movement, route choice, and resource pressure feel sane, replace enemy
silhouette placeholders with production direction models.

Order:

1. Burdened
2. Ashbound
3. Censer
4. Chainrunner
5. Harpy

Acceptance:

- silhouette matches `visual_tell`
- collider still fits
- manual smoke confirms readability during combat

## Phase 5: Texture Pass

Only after the geometry reads:

- anchor pale gold/stone
- resource glow
- hazard ember/ash
- route kit dark stone
- relic blue-green accent

Textures should support silhouettes, not rescue unclear models.

## Always Run

```powershell
powershell -ExecutionPolicy Bypass -File scripts/foundation_check.ps1
```

Manual check:

```powershell
cargo run -- foundation_test
```
