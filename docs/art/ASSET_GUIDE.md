# Cenotaph Asset Guide

## Purpose

This file defines the working rules for placeholder and production model assets.

The goal is to make every model easy to replace without breaking levels,
physics, validation, or runtime loading.

## Solo Production Strategy

Cenotaph's art pipeline must be realistic for a solo developer. Asset work
should emphasize reusable silhouettes, modular kits, strong material treatment,
and presentation systems rather than large quantities of bespoke modeling.

Prefer:

- Small reusable level kits that can form many routes.
- A few readable enemy bodies with modifier overlays, material variants, and
  behavior differences.
- A few weapon or relic silhouettes that can support many data-driven variants.
- Simple meshes elevated by fog, lighting, particles, audio, tint, texture,
  scale, animation role, and placement.
- Generated prototype textures and repeatable asset scripts when they reduce
  manual work without weakening identity.

Avoid:

- Asset plans that require production-quality models before gameplay roles are
  proven.
- One-off props that cannot be reused in at least one additional context.
- Detail-heavy meshes whose gameplay role is unclear at first-person distance.
- Art direction that depends on volume rather than composition and symbolism.

The target is not "low-poly because cheap." The target is deliberate, readable,
symbolic construction that a solo developer can maintain.

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
- OBJ/GLTF diffuse texture references are used when they resolve to a runtime
  texture. Level `base_material` and prop `surface_material` can override them.
- Material texture paths are relative to `textures/` and support nested folders.
- Surface materials expose RGB tint, UV tiling from `0.05` to `64`, and bounded
  emissive strength from `0` to `4`.

## Runtime Presentation Profiles

Prototype model appearance is intentionally separate from model geometry. Edit
`config/tuning.toml` under `[visuals.default_profile]` or
`[visuals.profiles.<name>]` to change:

- `tint`: per-model RGB color multiplier
- `texture`: a path beneath `textures/`
- `uv_scale`: texture tiling
- `emissive`: glow contribution from `0` to `4`
- `animation_role`: `0` static, `1` pickup, `2` enemy, `3` Anchor/gate, `4`
  hazard

Profiles match asset IDs and enemy types by name. A level prop’s
`surface_material` still wins when it explicitly provides a texture or tint.
Press the runtime reload action after editing the file to apply changes without
restarting the game. Invalid colors, paths, ranges, and animation roles are
reported by `cargo run -- validate` and block runtime reload.

The HUD uses the same approach under `[ui.hud]`. Its semantic tokens (`bone`,
`gold`, `cold`, `blood`, `ember`, and the panel colors) can be changed without
editing widget layout code.

## Scale And Orientation

- Use Y-up.
- Treat 1 unit as roughly 1 meter.
- Put the origin at the gameplay contact point:
  - floor center for props
  - feet/ground center for enemies
  - centerline for pickups
- Face authored forward along negative Z unless the asset has no facing.
- Keep placeholder assets readable with the neutral fallback material.

## Export Rules

- Triangulate before export.
- Apply transforms before export.
- Keep filenames lowercase snake_case.
- Avoid spaces in filenames and folder names.
- Keep a single gameplay-readable mesh per placeholder asset.
- Keep gameplay meaning in level/data fields rather than material names.
- Keep diffuse references portable; use a filename or a path beneath
  `textures/`, never an absolute source-machine path.

## Directory Rules

Use these folders for the scratch model pass:

```text
source_assets/maps/
source_assets/reference/
assets/blockout/
assets/props/
assets/enemies/
assets/level_kit/
assets/relics/
textures/
```

Keep editable Blender files, conversion inputs, and visual references under
`source_assets/`. Only runtime-ready exports belong under `assets/` or
`textures/`.

The generated prototype kit lives in `textures/cenotaph/` and can be rebuilt
with `scripts/generate_prototype_textures.ps1`.

The deterministic OBJ generator emits exactly one box, frustum, or octahedron
per runtime placeholder and gives it a stable skewed projection UV. It must not
invent compound silhouettes, anatomy, or ornament.

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

- its dimensions and motion remain readable at first-person distance
- it exposes gameplay role without claiming to be production art
- its origin makes placement predictable
- its approximate size matches the intended collider
- it passes `cargo run -- validate`
- it can be used in `levels/foundation_test.json` without special code
- it can be reused, recolored, or resized without adding geometry

## Replacement Rule

Do not delete old placeholders until the replacement:

- is referenced by at least one test level or data definition
- passes content validation
- renders in a manual smoke test
- has a stable asset ID that future levels can keep using

## Visual Replacement Rule

Do not design replacements from gameplay names or lore alone. Keep the single
primitive until the developer supplies a sketch, reference, authored model, or
explicit shape/material direction for that specific asset.
