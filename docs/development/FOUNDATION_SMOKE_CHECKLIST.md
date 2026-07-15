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
- [ ] `F1` toggles a stable FPS/frame-time and world-count panel without
  changing movement or resizing the normal HUD.
- [ ] `F5` reloads tuning, bindings, models, textures, enemy/relic definitions,
  and the current level together.
- [ ] A malformed file makes `F5` report a rejected reload while the current
  playable scene remains intact.
- [ ] HUD text is readable against bright and dark map surfaces; interaction
  prompts and dialogue appear contextually instead of occupying the screen at
  all times.
- [ ] `Escape` pauses, releases cursor, shows pause overlay, and resumes cleanly.
- [ ] Non-tonal wind/pressure ambience starts, pauses, and resumes with the game.
- [ ] Fire, hits, pickups, blocking, movement, rites, and death emit no
  synthesized one-shot tones while authored recordings are absent.

## First Ascent Loop

- [ ] The opening alcove is safe, readable, and cannot trap the player between
  its walls.
- [ ] Interacting with the Oath Stone fires its dialogue/resource event once;
  walking past it without interacting does not fire it.
- [ ] Dialogue advances automatically or one line per `E` press, and the same
  press does not also fire a second world interaction.
- [ ] Pale waystones make the compact climb readable without permanent HUD
  instructions or oversized test walls.
- [ ] The Ash Spike Crossing is visible before entering its damage radius.
- [ ] Trail resources remain unsecured until the summit Anchor and are lost on
  death before banking.
- [ ] The field relic pickup is readable and later pickups do not silently
  replace the equipped relic.
- [ ] The Ash-Warden has no distant enemy marker or named health bar; both
  appear only when its authored activation range begins combat.
- [ ] Defeating the named Ash-Warden spawns `Debt of the Last Keeper` in the
  world, removes its named encounter HUD, and triggers the authored red
  fog/light/ash mountain reaction.
- [ ] Oath Stone dialogue, Warden death, generated loot, and Anchor flags never
  place a literal `DEBUG` event in the live HUD.
- [ ] Claiming `Debt of the Last Keeper` shows its full name, rarity, and whether
  it was equipped or stored.
- [ ] Approaching the summit Anchor shows `COMMUNE WITH ANCHOR` but does not
  bind Ash automatically.
- [ ] `E` opens the Anchor Rite; movement keys change the selected rite without
  moving the player, held primary fire does not fire, and `E` confirms it.
- [ ] Attempting to bind the First Anchor before defeating the Ash-Warden shows
  the unmet Last Chain requirement and does not claim or autosave the Anchor.
- [ ] `BIND THE CINDERS` converts carried Ash to Bound Ash, sets the local
  respawn point, and triggers the cold first-claim mountain reaction once.
- [ ] Returning to an Anchor whose one-shot claim event already fired can bind
  it again without replaying the event or reporting an error.
- [ ] Binding immediately after the Warden falls lets both mountain reactions
  complete in order instead of replacing the first reaction.
- [ ] `MEND THE VESSEL` spends the configured Bound Ash only while wounded;
  insufficient Ash and an unwounded vessel do not spend anything.
- [ ] `TURN FROM THE STONE` closes the rite without changing resources or the
  active Anchor.
- [ ] The return gate loads `foundation_test` after the Anchor area is reached.

## Foundation Test Arena

Run:

```powershell
cargo run -- foundation_test
```

Pass checks:

- [ ] Decorative, resource, Anchor, static, enemy, hurtbox, and trigger props render.
- [ ] Walking over the small resource prop logs unsecured resource collection.
- [ ] Walking near the Anchor does not activate it; interacting and choosing
  `BIND THE CINDERS` claims it and binds carried Ash.
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
- [ ] Player death triggers visual feedback, delayed respawn, and health
  restoration without a synthesized death sting.
- [ ] Transition trigger loads the target level without panic.

## Controlled Content Validation

Run:

```powershell
cargo run -- validate
cargo run -- play foundation_test
```

Pass checks:

- [ ] Validation catches duplicate IDs, missing model assets, invalid loot-table
  relic IDs, unresolved paths/events/dialogues, invalid event actions, and
  malformed path waypoints; props with authored loot tables require stable IDs.
- [ ] `F5` transactionally reloads valid tuning, bindings, models, textures,
  registries, and current-level data without restarting the game.
- [ ] Invalid content is rejected with actionable console output while the
  currently loaded play state remains intact.
- [ ] An `OnEnter`, proximity, or interaction event can grant resources, spawn
  loot, present dialogue, set a level-local flag, or queue a transition.
- [ ] Once-only events do not refire after saving and continuing, and
  flag-gated events wait until their required level flag has been set.
- [ ] A prop or enemy with a valid `path_id` follows its authored path and
  stops or loops according to the path definition.

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
  file instead of replaying authored rewards on continue.
- [ ] Continuing after defeating the Ash-Warden does not revive it; an
  uncollected generated relic remains in the world, while a collected relic does
  not respawn.
- [ ] Moving an authored Anchor while keeping its `anchor_id` makes continue use
  the new position; removing that Anchor clears the binding and uses level spawn.
- [ ] Obsolete saved event, flag, reaction, relic, and removed-prop IDs are
  discarded once and absent from the compatibility-cleanup autosave.
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
