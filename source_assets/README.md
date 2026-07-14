# Source Assets

This folder contains editable or reference inputs that are not loaded directly
by the game runtime.

- `maps/` contains Blender files and authoring exports used to produce runtime
  map assets.
- `reference/` contains concept and visual reference images.
- Runtime-ready `.obj`, `.gltf`, and `.glb` files remain under `assets/`.
- Runtime textures remain under `textures/`.

The standalone level editor catalogs this folder so source files can be staged
through asset-import records without mixing them into runtime directories.
