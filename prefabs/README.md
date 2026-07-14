# Level Prefabs

Prefab JSON files are reusable, editor-authored groups of relative props.

- Keep each prefab at schema version `1`.
- Every prop needs a unique local `id`.
- Prop positions are relative to the placement origin.
- Level-local loot, path, dialogue, and event references are intentionally
  rejected because they cannot be resolved safely in every destination level.
- `cargo validate-content` validates every prefab and referenced runtime asset.

The standalone editor creates backups in `prefabs/.editor_backups/` before an
existing prefab is overwritten or deleted.
