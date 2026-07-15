# Cenotaph UI Style and Extension Contract

## Intent

The interface should feel like a sacred instrument made from black iron, bone,
ash, tarnished gold, and cold Anchor light. It may be strange in silhouette, but
never vague about gameplay state.

Persistent HUD elements stay quiet. Combat warnings, interaction prompts, and
run-changing events may briefly become brighter or larger, then recede.

## Architecture

- `src/systems/render/hud/theme.rs` owns semantic colors. Widgets request a
  meaning such as `gold`, `cold`, `blood`, or `bone_dim`; they do not invent a
  local palette.
- `src/systems/render/hud/layout.rs` owns responsive regions and physical scale.
  It currently reserves ascent, player, notification, interaction, dialogue,
  objective, boss, and status-effect space.
- `src/systems/render/hud.rs` owns reusable drawing primitives, sigils, text,
  meters, and widget composition.
- `HudFrameState` is the render boundary. Gameplay systems provide semantic
  state and never emit screen coordinates for persistent widgets.

## Visual Rules

- Bone is primary text; dim bone is metadata.
- Tarnished gold marks relics, rites, ascent identity, and important rewards.
- Cold light marks Anchors, interaction, and defensive states.
- Blood and ember are reserved for health, damage, danger, and attack timing.
- Use one dominant accent per widget. Keep secondary information quieter.
- Prefer hairline rails, cut corners, diamonds, and sigils over stacked boxes.
- Show a meter only when its continuous value matters. Show a number only when
  the exact value changes a decision.
- World markers identify; they do not duplicate a full HUD panel over an object.
- Enemy markers are combat telegraphs, not scouting sensors. They remain hidden
  until the enemy is inside its actual activation range.
- Do not shrink important text to solve layout. Wrap it, shorten it, or give the
  widget more room.
- Authored dialogue, flags, loot manifestation, and mountain answers never emit
  `DEBUG`, spawn-count, or reload feedback into the player-facing HUD.

## Adding a Widget

1. Add semantic data to `HudFrameState` or a focused child state structure.
2. Use an existing `HudLayout` region. Add a named region only when ownership is
   genuinely new, and include it in layout validation.
3. Compose `push_cut_panel`, `push_line`, `push_diamond`, `push_icon`, themed
   meters, keycaps, and UI text helpers before adding a new primitive.
4. Use `HudTextFit` for bounded labels and wrapping for sentence-length copy.
5. Test 800x600, 1280x720, 1920x1080, and 2560x1080. Persistent corner regions
   must remain on screen and must not overlap.
6. Capture the native game window with representative combat density before
   considering the widget complete.

## Planned Regions

- `objective`: current objective or route choice near the upper center.
- `boss`: encounter identity and health near the lower center.
- `status_effects`: compact timed effects above the player panel.

These regions are intentionally reserved but invisible until their systems have
real decisions to communicate.

At compact 4:3 sizes, the boss region moves into the available lower-right
space instead of covering the player panel. Dialogue moves above the player
panel. These are layout responsibilities, not per-widget exceptions.
