# Standalone Level Editor

The editor is launched through the authenticated local Rust server:

```powershell
cargo run -- editor
```

Open the tokenized URL printed in the terminal. The web files are embedded into
the executable at compile time, so restart the command after changing them.
`cargo editor` is the shorter equivalent once Cargo has read the project config.

- `index.html` defines the Hammer-style authoring workspace.
- `styles.css` owns the compact desktop layout and interaction states.
- `app.js` owns project API calls, level state, history, viewports, geometry,
  terrain sculpting, inspectors, and save/validation workflows.
- `prefab-tools.js` owns deterministic selection capture and relative group
  instantiation without server or DOM dependencies.
- `src/developer/editor_server.rs` owns localhost security, project discovery,
  validation, backups, level writes, and prefab lifecycle routes.

The editor uses the same level schema, validation, rendering mesh, and physics
mesh contracts as the game runtime.

Unsaved levels are debounced into browser-local drafts. Reloading or reopening
a level offers recovery, warns when its disk version changed, and puts the disk
snapshot one Undo step behind the recovered draft. A successful save or an
explicit discard clears the corresponding draft.

Prefab files live in `prefabs/`. The editor validates them through Rust before
writing, backs up overwrites/deletes, and expands each placement into ordinary
level props so the runtime needs no separate prefab loader.
