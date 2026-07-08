# Cenotaph Foundation Manual Smoke Checklist

Use this after `scripts/foundation_check.ps1` passes and before treating the
foundation build as stable.

The goal is not balance approval. The goal is to confirm that GPU launch,
input, render, audio, physics, and foundation combat still work together.

## Preflight

- [ ] Run `powershell -ExecutionPolicy Bypass -File scripts/foundation_check.ps1`.
- [ ] Confirm the script reports `Foundation check passed.`
- [ ] If console position logs are too noisy, set
  `debug.position_log_enabled = false` in `config/tuning.toml`.

## Ash-Walk Launch

Run:

```powershell
cargo run -- ashwalk_01
```

Pass checks:

- [ ] Window opens without panic.
- [ ] The Ash-Walk map renders; no blank screen or missing base map fallback.
- [ ] Mouse look updates the camera smoothly.
- [ ] `WASD` movement works and does not drift when released.
- [ ] `Space` jumps, falling returns to stable ground, and collision feels sane.
- [ ] `Shift` sprint changes speed and drains stamina on the HUD.
- [ ] `Q` dash fires once per press, consumes stamina, and respects cooldown.
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
- [ ] Death after collecting unbanked resource logs unsecured resource loss.
- [ ] Respawn uses the active Anchor after it has been activated.
- [ ] Static props block movement as expected.
- [ ] Left mouse primary fire can hit the enemy silhouette.
- [ ] Enemy health decreases and the enemy prop disappears when defeated.
- [ ] Enemy chases when inside activation range.
- [ ] Enemy attack wind-up/cooldown deals damage without rapid-fire damage spam.
- [ ] Hurtbox proximity damages the player at a readable interval.
- [ ] Player death triggers hit feedback, death audio, delayed respawn, and health
  restoration.
- [ ] Transition trigger loads the target level without panic.

## Failure Notes

For every failed check, record:

- level command used
- observed behavior
- console output around the failure
- GPU/driver details if launch or rendering failed
- whether `cargo run -- validate` still passes
