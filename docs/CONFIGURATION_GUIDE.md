# Cenotaph Configuration Guide

## Overview

This guide explains how to use the configuration-driven content creation system for Cenotaph. Every aspect of the game can be modified through TOML configuration files without touching the code.

## Configuration Structure

```
config/
├── enemies/           # Enemy types and behaviors
├── weapons/           # Weapon parts and assembly rules
├── loot/             # Loot tables and drop rates
├── levels/           # Level definitions and progression
├── audio/            # Sound configuration
├── gameplay/         # Progression and game rules
└── world/            # World structure and strata
```

## Adding New Content

### 1. Adding a New Enemy

**Step 1:** Copy the template
```bash
cp config/enemies/template.toml config/enemies/new_enemy.toml
```

**Step 2:** Edit the configuration
```toml
# config/enemies/new_enemy.toml
name = "New Enemy Name"
description = "Description of enemy behavior"
health = 150
damage = 30
# ... other stats
```

**Step 3:** Register the enemy
```toml
# config/enemies/enemy_types.toml
[new_enemy]
file = "new_enemy.toml"
unlock_level = 8
spawn_weight = 3
description = "Description of new enemy."
```

**Step 4:** Add assets
- Model: `assets/enemies/new_enemy.obj`
- Texture: `assets/enemies/new_enemy.png`
- Audio: `assets/audio/enemies/new_enemy.wav`

### 2. Adding a New Weapon Part

**Step 1:** Copy the template
```bash
cp config/weapons/template.toml config/weapons/new_part.toml
```

**Step 2:** Edit the configuration
```toml
# config/weapons/new_part.toml
[part_info]
name = "New Part Name"
type = "barrel"
rarity = "rare"
philosophy = "fragmentation"
faction = "schizoid"

[stats]
damage_modifier = 1.5
fire_rate_modifier = 0.8
# ... other stats
```

**Step 3:** Add to registry
```toml
# config/weapons/weapon_registry.toml
[part_categories]
barrel = [
    "existing_parts...",
    "new_part"  # Add your new part here
]
```

**Step 4:** Add assets
- Model: `assets/weapons/parts/new_part.obj`
- Texture: `assets/weapons/parts/new_part.png`

### 3. Adding a New Level

**Step 1:** Copy the template
```bash
cp config/levels/template.toml config/levels/new_level.toml
```

**Step 2:** Edit the configuration
```toml
# config/levels/new_level.toml
[level_info]
name = "New Level Name"
description = "Description of the level"
difficulty = 5
strata_type = "new_strata_type"
theme = "new_theme"
```

**Step 3:** Register the level
```toml
# config/levels/level_registry.toml
[new_level]
file = "new_level.toml"
unlock_level = 10
difficulty = 5
strata_order = 8
description = "Description of new level."
```

**Step 4:** Create level data
- JSON level file: `levels/new_level.json`
- Assets: `assets/levels/new_level/`

### 4. Adding New Loot

**Step 1:** Edit loot tables
```toml
# config/loot/loot_tables.toml
[new_loot_table]
items = [
    { item = "new_item", chance = 0.1, min_count = 1, max_count = 1 }
]
```

**Step 2:** Add item definitions
```toml
[consumables]
new_item = { name = "New Item", description = "Description", effect = "heal_50", rarity = "rare" }
```

## Configuration Categories

### Enemy Configuration

**Core Stats:**
- `health` - Enemy hit points
- `damage` - Attack damage
- `speed` - Movement speed
- `armor` - Damage reduction

**Behavior:**
- `aggression` - 1-10 scale of attack tendency
- `intelligence` - 1-10 scale of tactical behavior
- `patrol_radius` - Area of movement
- `alert_radius` - Detection range

**Design Document Integration:**
- `faction` - burdened, silencers, paranoiacs, harpies
- `philosophy` - physical_weight, anti_magic, area_denial, aerial_threats

### Weapon Configuration

**Part Types:**
- `barrel` - Fire rate, accuracy, damage modifiers
- `scope` - Zoom, accuracy, visual effects
- `stock` - Stability, recoil control
- `magazine` - Capacity, reload speed
- `trigger` - Fire rate, special effects
- `grip` - Handling, stability

**Faction Philosophies:**
- `schizoid` - Fragmentation, instability, high fire rate
- `moonchild` - Guidance, illumination, homing effects
- `sovereign` - Authority, control, gravity effects

### Level Configuration

**Vertical Design:**
- `vertical_range` - Min/max height for the level
- `primary_path_heights` - Key elevation points
- `shortcut_locations` - Vertical traversal points

**Strata System:**
- `strata_type` - Links to world structure
- `theme` - Visual and gameplay theme
- `palette` - Color scheme
- `traversal` - Movement mechanics

### Progression Configuration

**Level Progression:**
- `base_xp` - Experience required per level
- `multiplier` - Growth rate of experience requirements
- `stat_progression` - Stats gained per level

**Unlock System:**
- `new_weapons` - Levels when new weapon types unlock
- `new_abilities` - Levels when new abilities unlock
- `new_areas` - Levels when new areas become accessible

## Best Practices

### 1. Template-First Development
Always start with existing templates and modify them. This ensures consistency and reduces errors.

### 2. Incremental Testing
Add one piece of content at a time and test it before moving on to the next. This makes debugging much easier.

### 3. Asset Organization
Keep assets organized with clear naming conventions:
```
assets/
├── enemies/
│   └── enemy_name/
│       ├── model.obj
│       ├── texture.png
│       └── audio.wav
├── weapons/
│   └── weapon_part/
│       ├── model.obj
│       └── texture.png
└── levels/
    └── level_name/
        ├── level.json
        └── assets/
```

### 4. Configuration Validation
Always validate your TOML files using online validators or IDE extensions to catch syntax errors early.

### 5. Documentation
Update this guide when you add new configuration categories or change existing ones.

## Troubleshooting

### Common Issues

**Enemy Not Spawning:**
- Check that the enemy is registered in `enemy_types.toml`
- Verify the unlock level is reached
- Ensure spawn weights are set correctly

**Weapon Part Not Appearing:**
- Check that the part is added to the correct category in `weapon_registry.toml`
- Verify compatibility with weapon types
- Ensure rarity and unlock requirements are met

**Level Not Loading:**
- Check that the level is registered in `level_registry.toml`
- Verify the JSON level file exists
- Ensure all required assets are present

**Configuration Not Loading:**
- Check TOML syntax with a validator
- Ensure files are in the correct directory
- Verify file names match references

### Debug Tools

**Configuration Validation:**
```bash
# Validate TOML syntax
toml validate config/enemies/new_enemy.toml
```

**Asset Verification:**
```bash
# Check if required assets exist
ls assets/enemies/new_enemy.*
```

**Game Logs:**
Check the game logs for configuration loading errors and missing asset warnings.

## Future Expansion

### Adding New Configuration Categories

1. Create the directory: `config/new_category/`
2. Create a template file: `config/new_category/template.toml`
3. Add loading code to the configuration manager
4. Update this documentation

### Configuration Hot-Reloading

The system supports hot-reloading of configuration files during development. Changes to TOML files will be automatically detected and loaded without restarting the game.

### Mod Support

This configuration system is designed to support modding. External configuration files can be loaded from:
```
mods/
├── mod_name/
│   ├── config/
│   │   ├── enemies/
│   │   ├── weapons/
│   │   └── levels/
│   └── assets/
```

## Examples

### Complete Enemy Example
```toml
# config/enemies/example_enemy.toml
name = "Example Enemy"
description = "An example enemy for testing"
health = 100
damage = 25
speed = 3.0
faction = "burdened"
philosophy = "physical_weight"
```

### Complete Weapon Part Example
```toml
# config/weapons/example_part.toml
[part_info]
name = "Example Barrel"
type = "barrel"
rarity = "common"
philosophy = "fragmentation"
faction = "schizoid"

[stats]
damage_modifier = 1.2
fire_rate_modifier = 0.9
accuracy_modifier = 0.8
```

### Complete Level Example
```toml
# config/levels/example_level.toml
[level_info]
name = "Example Level"
description = "An example level for testing"
difficulty = 3
strata_type = "ash_walk"
theme = "collapse"

[spawn_points]
default = [0.0, 10.0, 0.0]
```

This configuration system makes Cenotaph infinitely expandable while keeping development simple and accessible for solo developers.