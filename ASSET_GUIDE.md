# Cenotaph Asset Guide

## Purpose

This file defines the working rules for placeholder and production model assets.

The goal is to make every model easy to replace without breaking levels,
physics, validation, or runtime loading.

## Current Engine Contract

- Supported model formats: `.obj`, `.glb`, `.gltf`.
- Props reference models by `asset_id` relative to `assets/`.
  - Example: `props/anchor_placeholder.obj`.
- Base maps reference full paths.
  - Example: `assets/ashwalk_001.obj`.
- Model assets must contain vertices, render indices, physics points, and
  physics triangles.
- Prop collision is controlled by level JSON through `collider_type`.
- Enemy model, collider, health, and combat values come from
  `data/enemies/*.toml`.
- The renderer currently uses the default texture path for loaded model parts,
  so shape readability matters more than materials.

## Scale And Orientation

- Use Y-up.
- Treat 1 unit as roughly 1 meter.
- Put the origin at the gameplay contact point:
  - floor center for props
  - feet/ground center for enemies
  - centerline for pickups
- Face authored forward along negative Z unless the asset has no facing.
- Keep placeholder assets readable in untextured gray.

## Export Rules

- Triangulate before export.
- Apply transforms before export.
- Keep filenames lowercase snake_case.
- Avoid spaces in filenames and folder names.
- Keep a single gameplay-readable mesh per placeholder asset.
- Do not rely on material names for gameplay meaning yet.

## Directory Rules

Use these folders for the scratch model pass:

```text
assets/blockout/
assets/props/
assets/enemies/
assets/level_kit/
assets/relics/
textures/
```

Recommended asset IDs:

```text
props/anchor_placeholder.obj
props/resource_shard.obj
props/hazard_ash_spike.obj
level_kit/route_platform.obj
level_kit/route_ramp.obj
level_kit/route_gate.obj
relics/relic_placeholder.obj
props/cache_placeholder.obj
props/safe_room_marker.obj
```

## Placeholder Quality Bar

A placeholder is good enough when:

- its silhouette reads from first-person distance
- its origin makes placement predictable
- its approximate size matches the intended collider
- it passes `cargo run -- validate`
- it can be used in `levels/foundation_test.json` without special code

## Replacement Rule

Do not delete old placeholders until the replacement:

- is referenced by at least one test level or data definition
- passes content validation
- renders in a manual smoke test
- has a stable asset ID that future levels can keep using

## First Scratch Asset Batch

Build these first:

- Anchor pillar
- Resource shard
- Hazard marker
- Route platform
- Route ramp
- Route gate
- Relic pickup
- Cache or reward container

Enemies can remain low-poly silhouettes until the route/resource loop is fun.
