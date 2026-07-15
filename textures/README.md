# Textures Directory

Place PNG or JPG texture files in this directory. Nested folders are supported;
materials reference paths relative to this directory.

The engine scans this directory on boot and uploads supported images to the GPU.
Textures can come from OBJ/GLTF material references or explicit level and prop
surface materials. Missing references use a neutral fallback.

## Notes
- `cenotaph/` contains the small generated prototype kit.
- Regenerate it with `scripts/generate_prototype_textures.ps1`.
- Supported formats: PNG, JPG/JPEG, WebP, BMP, and TGA.
