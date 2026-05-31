# Textures Directory

Place PNG or JPG texture files in this directory. They will be automatically
loaded by the engine at startup and mapped to mesh parts by filename.

The engine scans this directory on boot and uploads all `.png` and `.jpg` files
to the GPU as texture resources. To assign a texture to a prop, name the file
the same as the `texture_name` field in your prop definitions (or use `"default"`
as a fallback).

## Notes
- Textures are gitignored (they're large binary files)
- Supported formats: PNG, JPG/JPEG
- If no textures are found, a checkerboard fallback is shown