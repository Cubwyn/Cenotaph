# Level Prefabs

Prefab JSON files are reusable, hand-authored groups of relative props.

- Keep each prefab at schema version `1`.
- Every prop needs a unique local `id`.
- Prop positions are relative to the placement origin.
- Level-local loot, path, dialogue, and event references are intentionally
  rejected because they cannot be resolved safely in every destination level.
- `cargo validate-content` validates every prefab and referenced runtime asset.

Keep prefab changes small and validate the project after editing them.
