# Cenotaph Coding Agent Instructions

Before making design-affecting code, content, UI, economy, combat, loot, level,
or art-direction changes, read:

- `docs/design/IDENTITY_CONTRACT.md`
- `docs/design/LORE.md`
- `docs/design/CONTENT_GUIDE.md`

Before changing `ashwalk_01`, its Anchor, named encounter, reward, or mountain
reaction flow, also read `docs/design/ASH_WALK_PILGRIMAGE.md`.

The owner-supplied gameplay reset is recorded in
`docs/design/GAMEPLAY_IDENTITY_RESET.md`. It supersedes conflicting gameplay
interpretations in older planning text while preserving the established lore,
Strata, terminology, UI identity, and art direction.

The target structure is an authored campaign followed by optional New Game+
and optional endgame replay. Cycles are not the primary progression structure.
Do not implement Cycle resets, death consequences, NG+ persistence, equipment
slots, weapon-generation fields, or other major progression rules when the
reset marks them UNDEFINED. Record the exact design question and stop instead
of filling the gap with genre convention.

For event-linked enemy defeats and Anchor claims, commit the mechanical change
and its authored event consequence in the same save. Authored loot sources need
stable prop IDs. Named encounters, Anchors, and relics must retain authored
display names at their HUD surfaces.

The identity contract is the top-level guardrail. If a requested implementation
would make Cenotaph feel like a Tarkov-style extraction shooter, generic
low-poly shooter, practical survival raid, or market/stash economy, stop and
offer a Cenotaph-aligned version.

Default project framing:

- The game is a surreal decayed-mountain RPG looter shooter.
- The core loop is ascent, ritual combat, cursed loot, Anchors, world/self
  change, and deeper ascent.
- The mountain is the protagonist.
- Loot is memory, survival, temptation, and buildcraft, not vendor trash.
- Anchors are ritual checkpoints, not extraction points.
- Runs are ascents, cycles, pilgrimages, echoes, or coronation failures, not
  raids.
- Runtime placeholder models are deliberately neutral single primitives. Coding
  agents must not invent anatomy, ornament, silhouettes, weapons, relic forms,
  or production-looking models from names or lore. Keep each placeholder to one
  box, frustum, or octahedron until the user supplies visual direction or a
  reference.
- Do not synthesize one-shot bleeps, chimes, tones, bells, hit noises, or UI
  confirmations. Silence is preferable to fake feedback. Semantic sound hooks
  may remain for future authored recordings; placeholder ambience must remain
  non-tonal wind or pressure noise only.
- Cenotaph is a solo-developed project. Prefer reusable assets, procedural or
  data-driven gameplay variation, strong validation, and programming-heavy
  systems over plans that require large volumes of bespoke modeling, animation,
  cinematic content, or hand-authored map detail. This does not authorize AI to
  invent visual or audio content.

When in doubt, choose the option that makes the Cenotaph stranger, taller,
older, more dangerous, and more mechanically expandable.
