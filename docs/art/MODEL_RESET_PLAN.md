# Cenotaph Primitive Placeholder Policy

## Purpose

The runtime uses deliberately neutral geometry while Cenotaph's visual forms
remain developer-directed. Placeholder models must expose gameplay scale,
collision, material, movement, and placement without pretending to solve enemy,
relic, Anchor, hazard, or architectural design.

## Hard Rule

Every model under these runtime directories is exactly one basic primitive:

- `assets/enemies/`
- `assets/pickups/`
- `assets/props/`
- `assets/world/`

Allowed placeholder forms are a box, frustum, or octahedron. A placeholder may
change dimensions, tint, emissive role, motion, or authored scale. It must not
combine primitives or add limbs, wings, chains, rings, faces, weapons, clothing,
ornament, damage, asymmetry, or lore-derived iconography.

`cargo run -- validate` enforces one mesh part, no more than eight physics
vertices, and no more than twelve render or physics triangles for these files.

## Current Mapping

- Enemies use boxes or one octahedron with role-specific proportions.
- Relics and resource pickups use one octahedron each.
- The Oath Stone uses one frustum.
- Platforms, walls, and the transition marker use one box each.
- The Anchor and hurtbox marker use one octahedron each.

These forms are not canon. Their proportions are debugging information, not
permission to infer anatomy or production art.

## Readability

Until real assets exist, communicate gameplay through:

- dimensions and scale
- role tint and material
- speed, elevation, pathing, wind-up, and stagger
- particles, fog, lighting, and encounter framing
- names and HUD state where appropriate

Do not add geometry to solve readability.

## Replacement Gate

A primitive may be replaced only when the developer supplies a reference,
sketch, model, or explicit visual direction for that asset. Replacement work
must preserve its stable runtime path unless an intentional migration is made.
An AI coding pass must not generate or commission the replacement on its own.

## Regeneration

After changing primitive dimensions, rebuild the kit with:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/generate_prototype_models.ps1
```

Then run:

```powershell
cargo run -- validate
```
