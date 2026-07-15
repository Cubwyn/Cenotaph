"""Generate Cenotaph's small deterministic prototype texture kit.

Only the Python standard library is used, keeping the authored materials
reproducible on a clean solo-development machine.
"""

from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "textures" / "cenotaph"
SIZE = 128


def clamp_byte(value: float) -> int:
    return max(0, min(255, round(value)))


def hash2(x: int, y: int, seed: int) -> float:
    value = (x * 0x1F123BB5) ^ (y * 0x5F356495) ^ (seed * 0x9E3779B9)
    value = (value ^ (value >> 16)) * 0x45D9F3B
    value = (value ^ (value >> 16)) * 0x45D9F3B
    value ^= value >> 16
    return (value & 0xFFFFFFFF) / 0xFFFFFFFF


def smooth(value: float) -> float:
    return value * value * (3.0 - 2.0 * value)


def value_noise(x: float, y: float, cell: int, seed: int) -> float:
    cells = SIZE // cell
    gx = math.floor(x / cell)
    gy = math.floor(y / cell)
    tx = smooth((x / cell) - gx)
    ty = smooth((y / cell) - gy)
    a = hash2(gx % cells, gy % cells, seed)
    b = hash2((gx + 1) % cells, gy % cells, seed)
    c = hash2(gx % cells, (gy + 1) % cells, seed)
    d = hash2((gx + 1) % cells, (gy + 1) % cells, seed)
    top = a + (b - a) * tx
    bottom = c + (d - c) * tx
    return top + (bottom - top) * ty


def fractal(x: float, y: float, seed: int) -> float:
    return (
        value_noise(x, y, 32, seed) * 0.50
        + value_noise(x, y, 16, seed + 1) * 0.30
        + value_noise(x, y, 8, seed + 2) * 0.20
    )


def png_chunk(kind: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
    )


def write_png(path: Path, pixel_fn) -> None:
    rows = bytearray()
    for y in range(SIZE):
        rows.append(0)
        for x in range(SIZE):
            rows.extend(clamp_byte(channel) for channel in pixel_fn(x, y))
    header = struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0)
    payload = (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", header)
        + png_chunk(b"IDAT", zlib.compress(bytes(rows), 9))
        + png_chunk(b"IEND", b"")
    )
    path.write_bytes(payload)


def ash_stone(x: int, y: int):
    grain = fractal(x, y, 11)
    strata = math.sin(y * math.tau / 32.0 + value_noise(x, y, 16, 17) * 2.2)
    crack = abs(math.sin(x * 0.105 + math.sin(y * 0.073) * 2.4)) < 0.035
    base = 62 + grain * 42 + strata * 5 - (28 if crack else 0)
    fleck = 16 if hash2(x, y, 93) > 0.988 else 0
    return base + fleck, base + fleck * 0.8, base - 3, 255


def weathered_stone(x: int, y: int):
    row = y // 24
    shifted_x = (x + (16 if row % 2 else 0)) % 32
    seam = shifted_x < 2 or y % 24 < 2
    grain = fractal(x, y, 31)
    moss = max(0.0, value_noise(x, y, 16, 38) - 0.68) * 38
    base = 82 + grain * 42 - (30 if seam else 0)
    return base - moss * 0.35, base + moss * 0.45, base + moss * 0.18, 255


def black_iron(x: int, y: int):
    grain = fractal(x, y, 51)
    brushed = math.sin(x * math.tau / 9.0) * 3 + math.sin(y * math.tau / 41.0) * 4
    seam = x % 64 < 2
    rx = min(x % 32, 32 - x % 32)
    ry = min(y % 32, 32 - y % 32)
    rivet = math.hypot(rx, ry) < 2.3
    base = 31 + grain * 25 + brushed - (13 if seam else 0)
    if rivet:
        base += 48
    return base + 4, base + 6, base + 8, 255


def ember_cracks(x: int, y: int):
    grain = fractal(x, y, 71)
    vein = abs(math.sin(x * 0.087 + math.sin(y * 0.061) * 3.7 + grain * 2.0))
    glow = max(0.0, 1.0 - vein / 0.085)
    coal = 20 + grain * 25
    return coal + glow * 225, coal * 0.72 + glow * 78, coal * 0.60 + glow * 14, 255


def pale_waystone(x: int, y: int):
    grain = fractal(x, y, 101)
    rings = math.sin((x + y) * math.tau / 47.0 + grain * 1.7) * 5
    pore = 18 if hash2(x, y, 112) > 0.992 else 0
    base = 112 + grain * 46 + rings - pore
    return base * 0.78, base * 0.94, base, 255


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    textures = {
        "ash_stone.png": ash_stone,
        "weathered_stone.png": weathered_stone,
        "black_iron.png": black_iron,
        "ember_cracks.png": ember_cracks,
        "pale_waystone.png": pale_waystone,
    }
    for filename, pixel_fn in textures.items():
        path = OUTPUT / filename
        write_png(path, pixel_fn)
        print(f"generated {path.relative_to(ROOT).as_posix()}")


if __name__ == "__main__":
    main()
