# Ash-Walk Pilgrimage Slice

## Status

This is the current playable identity milestone for Cenotaph.

Do not add another Stratum until this slice is manually playable, readable, and
worth repeating. Its purpose is to prove that Cenotaph feels like a surreal
ascent with cursed buildcraft, not a raid followed by extraction.

## Player Experience

```text
Enter the Ash-Walk
-> accept or ignore the Oath Stone
-> carry exposed Ash through ritual combat
-> claim the Ash Splinter
-> confront the Ash-Warden, Bearer of the Last Chain
-> witness the mountain recoil
-> claim Debt of the Last Keeper
-> commune with the First Anchor
-> bind, mend, or turn away
-> continue the ascent
```

The route is intentionally small. Atmosphere, encounter identity, named rewards,
and systemic reactions must create the sense of a much larger mountain.

## Anchor Rite

Anchors never activate by proximity. The player must interact and choose:

- **Bind the Cinders:** carried Ash becomes Bound Ash, the Anchor becomes the
  respawn claim, and its first-claim event makes the mountain answer. At the
  First Anchor this choice remains unavailable until the Last Chain is broken.
- **Mend the Vessel:** spend Bound Ash to restore health. Failure is atomic:
  an unwounded vessel or insufficient balance spends nothing.
- **Turn from the Stone:** close the rite without changing the ascent.

The rite suspends movement, enemy updates, pickups, and weapon input while open.
It is a ritual decision, not an inventory or extraction screen.

## Named Encounter

The upper blocker is **The Ash-Warden, Bearer of the Last Chain**, a dedicated
enemy definition that reuses the Burdened body while changing scale,
presentation, stats, name, and authored context.

Its authored name and health appear when the player enters its real activation
range and remain visible while the keeper is an active threat. Neither the
named-encounter HUD nor the enemy world marker may reveal it from elsewhere in
the Stratum. The reused body is production leverage, not permission to make the
encounter anonymous.

Its guaranteed drop is **Debt of the Last Keeper**:

> The Anchor forgave its bearer. The mountain did not.

Claiming the relic presents its name, rarity, and whether it was equipped or
stored. A named reward must never collapse into a generic loot notification.
Manifesting its drop does not emit developer feedback or a generic loot-count
banner.

The current relic schema supports a deliberate close-range damage/stagger
tradeoff. A future behavior-trait system should deepen this identity without
requiring a new model.

## Mountain Answers

Props can queue authored manual events through `event_id`. Events can invoke a
validated `ReactMountain` profile that temporarily changes clear color, fog,
key light, ash color and motion, wind, and non-tonal ambience pressure. Mountain
answers do not play synthesized one-shot cues.

Ash-Walk currently uses two answers:

- **Last Chain Severed:** the Warden's death closes the fog, gutters the light,
  reverses the ash, and lets the road acknowledge the missing weight.
- **First Claim Bound:** the First Anchor answers with cold light, rising ash,
  and a different ambience after the player's first valid claim.

The authored level atmosphere remains immutable. Reactions blend over it and
restore it exactly. Concurrent answers queue and play in authored order rather
than truncating one another.

## Persistence Contract

Continue reconstructs the current pilgrimage from a small world-state journal:

- stable authored prop IDs already defeated or collected
- generated loot that still exists in the world
- fired one-shot events and level-local flags
- active or queued mountain answers that have not completed

Enemy defeat and Anchor claim events commit their mechanical state and authored
consequence in the same autosave. A crash before that commit may require the
player to repeat the action; it must never preserve a dead Warden without its
mountain answer or an active Anchor without its first-claim rite.

Generated loot IDs are deterministic and idempotent. Continue ignores reaction
profiles, event state, flags, relics, and consumed-prop records removed by a
later content revision, then rewrites the save without those stale references.
Restored relic pickups use the current relic definition's pickup model rather
than preserving obsolete visual asset paths. Active Anchors resolve by stable
ID to their current authored position; a retired Anchor safely falls back to the
level spawn.

## Acceptance Gate

Before expanding breadth, verify:

- The climb reads as an ascent rather than a loot arena.
- The Ash-Warden is legible as a named keeper despite reused geometry.
- Its drop feels like a specific relic, not generic gear score.
- Both mountain answers are visually legible without obscuring combat; any
  audible change comes only from continuous wind/pressure ambience.
- If triggered close together, both mountain answers complete in order.
- Anchor choices are understandable and cannot spend resources accidentally.
- Death before binding feels costly; binding feels like a ritual claim.
- Defeated enemies, collected props, and loose generated loot remain coherent
  through validation, reload, death, and resume checks.

## Next Priority

Manually play and tune this slice. Then add one meaningful route split and one
behavior-changing relic or perk decision. Do not answer weak replayability with
more map, more generic loot, or more placeholder enemies.
