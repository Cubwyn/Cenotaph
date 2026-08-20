use super::*;

#[test]
fn level_ids_reject_path_traversal() {
    assert!(validate_level_id("movement_test").is_ok());
    assert!(validate_level_id("../movement_test").is_err());
    assert!(validate_level_id("nested/level").is_err());
}

#[test]
fn default_level_has_correct_name() {
    let level = LevelData::default_level();
    assert_eq!(level.name, "map_001");
    assert_eq!(level.version, CURRENT_LEVEL_VERSION);
}

#[test]
fn legacy_level_json_migrates_to_current_version() {
    let level = LevelData::from_json_str(
        r#"{
            "name": "Legacy",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": []
        }"#,
    )
    .unwrap();

    assert_eq!(level.version, CURRENT_LEVEL_VERSION);
    assert_eq!(level.validate(), Ok(()));
}

#[test]
fn future_level_version_is_rejected_before_runtime() {
    let error = LevelData::from_json_str(
        r#"{
            "version": 2,
            "name": "Future",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": []
        }"#,
    )
    .unwrap_err();

    assert!(error.contains("newer than supported version"));
}

#[test]
fn default_level_starts_empty() {
    let level = LevelData::default_level();
    assert!(level.props.is_empty());
}

#[test]
fn terrain_brush_metadata_round_trips() {
    let geometry = BrushGeometryData {
        kind: Some("terrain".to_string()),
        terrain: Some(TerrainBrushData {
            columns: 2,
            rows: 2,
            seed: 17,
            relief: 3.0,
            base_thickness: 0.5,
            sculpt_strength: 0.75,
        }),
        vertices: vec![[0.0, 0.0, 0.0]; 18],
        faces: vec![[0, 1, 2]],
    };

    let json = serde_json::to_string(&geometry).unwrap();
    let loaded: BrushGeometryData = serde_json::from_str(&json).unwrap();
    let terrain = loaded.terrain.unwrap();

    assert_eq!(loaded.kind.as_deref(), Some("terrain"));
    assert_eq!(terrain.columns, 2);
    assert_eq!(terrain.rows, 2);
    assert_eq!(terrain.seed, 17);
    assert_eq!(terrain.sculpt_strength, 0.75);
}

#[test]
fn terrain_brush_validation_rejects_invalid_grid_metadata() {
    let geometry = BrushGeometryData {
        kind: Some("terrain".to_string()),
        terrain: Some(TerrainBrushData {
            columns: 1,
            rows: 30,
            seed: 0,
            relief: 1.0,
            base_thickness: 0.5,
            sculpt_strength: 0.5,
        }),
        vertices: vec![[0.0, 0.0, 0.0]; 3],
        faces: vec![[0, 1, 2]],
    };

    assert!(geometry
        .validation_errors("prop 0")
        .iter()
        .any(|error| error.contains("terrain grid must use between 2 and 24")));
}

#[test]
fn minimal_prop_json_uses_foundation_defaults() {
    let prop: PropData = serde_json::from_str(r#"{ "asset_id": "Cube.obj" }"#).unwrap();
    assert_eq!(prop.position, [0.0, 0.0, 0.0]);
    assert_eq!(prop.scale, [1.0, 1.0, 1.0]);
    assert_eq!(prop.collider_type, ColliderType::None);
    assert!(!prop.is_hurtbox);
    assert_eq!(prop.resource_value, 0);
    assert!(prop.anchor_id.is_none());
}

#[test]
fn prop_rotation_contract_is_degrees() {
    let prop: PropData =
        serde_json::from_str(r#"{ "asset_id": "Cube.obj", "rotation": [0.0, 90.0, 180.0] }"#)
            .unwrap();
    let radians = prop.rotation_radians();
    assert_eq!(radians[0], 0.0);
    assert!((radians[1] - std::f32::consts::FRAC_PI_2).abs() < f32::EPSILON);
    assert!((radians[2] - std::f32::consts::PI).abs() < f32::EPSILON);
}

#[test]
fn validation_allows_enemy_health_to_come_from_definition() {
    let prop: PropData =
        serde_json::from_str(r#"{ "asset_id": "Cube.obj", "enemy_type": "Burdened" }"#).unwrap();

    let errors = prop.validation_errors(0);
    assert!(errors.is_empty(), "{:?}", errors);
}

#[test]
fn validation_rejects_empty_anchor_id() {
    let prop: PropData =
        serde_json::from_str(r#"{ "asset_id": "Cube.obj", "anchor_id": "" }"#).unwrap();

    let errors = prop.validation_errors(0);
    assert!(errors.iter().any(|error| error.contains("anchor_id")));
}

#[test]
fn validation_requires_unique_authoring_safe_anchor_ids() {
    let level: LevelData = serde_json::from_str(
        r#"
        {
            "name": "Anchor Identity Test",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": [
                { "asset_id": "Cube.obj", "anchor_id": "same_anchor" },
                { "asset_id": "Cube.obj", "anchor_id": "same_anchor" },
                { "asset_id": "Cube.obj", "anchor_id": "unsafe anchor" }
            ]
        }
        "#,
    )
    .unwrap();

    let errors = level.validation_errors();
    assert!(errors
        .iter()
        .any(|error| error.contains("duplicate anchor id 'same_anchor'")));
    assert!(errors
        .iter()
        .any(|error| error.contains("anchor_id 'unsafe anchor' must use only")));
}

#[test]
fn prop_events_are_reserved_for_manual_enemy_or_anchor_consequences() {
    let level: LevelData = serde_json::from_str(
        r#"
        {
            "name": "Prop Event Contract Test",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": [
                { "id": "stone", "asset_id": "Cube.obj", "event_id": "stone_event" }
            ],
            "events": [
                {
                    "id": "stone_event",
                    "once": true,
                    "trigger": { "kind": "Proximity" },
                    "actions": [{ "kind": "GrantResource", "resource_value": 1 }]
                }
            ]
        }
        "#,
    )
    .unwrap();

    let errors = level.validation_errors();
    assert!(errors
        .iter()
        .any(|error| error.contains("only supported for enemy defeat or Anchor binding")));
    assert!(errors
        .iter()
        .any(|error| error.contains("must reference a Manual event")));
}

#[test]
fn validation_rejects_empty_item_id() {
    let prop: PropData =
        serde_json::from_str(r#"{ "asset_id": "Cube.obj", "item_id": "" }"#).unwrap();

    let errors = prop.validation_errors(0);
    assert!(errors.iter().any(|error| error.contains("item_id")));
}

#[test]
fn validation_rejects_enemy_resource_combo() {
    let prop: PropData = serde_json::from_str(
        r#"{ "asset_id": "Cube.obj", "enemy_type": "Burdened", "resource_value": 5 }"#,
    )
    .unwrap();

    let errors = prop.validation_errors(0);
    assert!(errors.iter().any(|error| error.contains("resource pickup")));
}

#[test]
fn validation_rejects_item_resource_combo() {
    let prop: PropData = serde_json::from_str(
        r#"{ "asset_id": "Cube.obj", "item_id": "ash_splinter", "resource_value": 5 }"#,
    )
    .unwrap();

    let errors = prop.validation_errors(0);
    assert!(errors.iter().any(|error| error.contains("item pickup")));
}

#[test]
fn authored_loot_sources_require_stable_prop_ids() {
    let prop: PropData = serde_json::from_str(
        r#"{ "asset_id": "Cube.obj", "enemy_type": "Burdened", "loot_table_id": "drop" }"#,
    )
    .unwrap();

    let errors = prop.validation_errors(0);

    assert!(errors
        .iter()
        .any(|error| error.contains("stable id when loot_table_id is set")));
}

#[test]
fn authored_props_cannot_claim_the_runtime_loot_namespace() {
    let prop: PropData =
        serde_json::from_str(r#"{ "id": "runtime_loot_authored", "asset_id": "Cube.obj" }"#)
            .unwrap();

    assert!(prop
        .validation_errors(0)
        .iter()
        .any(|error| error.contains("reserved 'runtime_loot_'")));
}

#[test]
fn event_linked_props_require_stable_ids() {
    let prop: PropData =
        serde_json::from_str(r#"{ "asset_id": "Cube.obj", "event_id": "keeper_fall" }"#).unwrap();

    assert!(prop
        .validation_errors(0)
        .iter()
        .any(|error| error.contains("stable id when event_id is set")));
}

#[test]
fn repeatable_automatic_events_are_rejected_before_they_can_fire_every_frame() {
    let mut level = LevelData::default_level();
    level.events = vec![LevelEventData {
        id: "repeat_proximity".to_string(),
        once: false,
        trigger: LevelEventTriggerData {
            kind: LevelEventTriggerKind::Proximity,
            position: [0.0, 0.0, 0.0],
            radius: 2.5,
            prop_id: None,
            flag_id: None,
        },
        actions: vec![LevelEventActionData {
            kind: LevelEventActionKind::GrantResource,
            target_level_id: None,
            loot_table_id: None,
            dialogue_id: None,
            reaction_id: None,
            flag_id: None,
            resource_value: 1,
            spawn_position: None,
        }],
    }];

    assert!(level
        .validation_errors()
        .iter()
        .any(|error| error.contains("repeatable automatic triggers are unsupported")));
}

#[test]
fn foundation_test_level_validates() {
    let level = LevelData::try_load("levels/foundation_test.json").unwrap();
    assert_eq!(level.props.len(), 16);
    assert_eq!(level.validate(), Ok(()));
}

#[test]
fn movement_test_level_validates() {
    let level = LevelData::try_load("levels/movement_test.json").unwrap();
    assert_eq!(level.props.len(), 18);
    assert_eq!(level.validate(), Ok(()));
}

#[test]
fn ashwalk_first_ascent_keeps_its_playable_loop_contract() {
    let level = LevelData::try_load("levels/ashwalk_01.json").unwrap();

    assert_eq!(level.version, CURRENT_LEVEL_VERSION);
    assert!(level.props.len() <= 14);
    assert_eq!(level.validate(), Ok(()));
    assert_eq!(level.atmosphere.particle_preset, ParticlePreset::Ashfall);
    assert_eq!(level.atmosphere.ambience_preset, AmbiencePreset::AshWind);
    assert_eq!(
        level.base_material.texture.as_deref(),
        Some("cenotaph/ash_stone.png")
    );
    assert_eq!(
        level
            .props
            .iter()
            .filter(|prop| prop.anchor_id.is_some())
            .count(),
        1
    );
    assert_eq!(level.props.iter().filter(|prop| prop.is_hurtbox).count(), 1);
    assert!(
        level
            .props
            .iter()
            .map(|prop| prop.resource_value)
            .sum::<u32>()
            >= 100
    );
    assert!(level
        .props
        .iter()
        .filter_map(|prop| prop.item_id.as_deref())
        .any(|item_id| item_id == "ash_splinter"));
    assert_eq!(level.paths.len(), 1);
    assert!(level
        .props
        .iter()
        .all(|prop| prop.asset_id != "props/test_wall.obj"));
    assert!(
        level
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some())
            .count()
            <= 3
    );
    assert!(level.events.iter().any(|event| {
        event.trigger.kind == LevelEventTriggerKind::Interact
            && event.trigger.prop_id.as_deref() == Some("oath_stone")
    }));

    let elite = level
        .props
        .iter()
        .find(|prop| prop.id.as_deref() == Some("ashwarden_elite"))
        .unwrap();
    assert!(elite.enemy_health >= 200.0);
    assert_eq!(elite.loot_table_id.as_deref(), Some("ashwarden_drop"));
    assert!(level
        .props
        .iter()
        .any(|prop| { prop.trigger_level_id.as_deref() == Some("foundation_test") }));
}

#[test]
fn level_save_round_trips_pretty_json() {
    let temp_path = std::env::temp_dir().join(format!(
        "cenotaph_level_save_test_{}_{}.json",
        std::process::id(),
        17
    ));
    let level = LevelData {
        version: CURRENT_LEVEL_VERSION,
        name: "Save Test".to_string(),
        base_map: "assets/test_movement_arena.obj".to_string(),
        player_spawn: [0.0, 128.0, 0.0],
        atmosphere: AtmosphereData::default(),
        base_material: SurfaceMaterialData::default(),
        mountain_reactions: Vec::new(),
        props: vec![PropData {
            id: None,
            display_name: None,
            asset_id: "props/test_wall.obj".to_string(),
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            collider_type: ColliderType::Box,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id: None,
            resource_value: 0,
            anchor_id: None,
            enemy_type: None,
            enemy_health: 0.0,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            loot_table_id: None,
            path_id: None,
            dialogue_id: None,
            event_id: None,
        }],
        asset_imports: Vec::new(),
        loot_tables: vec![LootTableData {
            id: "starter_loot".to_string(),
            rolls: 1,
            entries: vec![LootEntryData {
                weight: 1,
                item_id: Some("ash_splinter".to_string()),
                resource_value: 0,
                quantity: 1,
            }],
        }],
        paths: vec![LevelPathData {
            id: "enemy_patrol".to_string(),
            kind: LevelPathKind::Enemy,
            looped: true,
            speed_multiplier: 1.0,
            waypoints: vec![[0.0, 128.0, 0.0], [4.0, 128.0, 0.0]],
        }],
        events: vec![LevelEventData {
            id: "arrival_bark".to_string(),
            once: true,
            trigger: LevelEventTriggerData {
                kind: LevelEventTriggerKind::OnEnter,
                position: [0.0, 0.0, 0.0],
                radius: 2.5,
                prop_id: None,
                flag_id: None,
            },
            actions: vec![LevelEventActionData {
                kind: LevelEventActionKind::StartDialogue,
                target_level_id: None,
                loot_table_id: None,
                dialogue_id: Some("opening".to_string()),
                reaction_id: None,
                flag_id: None,
                resource_value: 0,
                spawn_position: None,
            }],
        }],
        dialogues: vec![DialogueData {
            id: "opening".to_string(),
            speaker: "Cenotaph".to_string(),
            lines: vec!["The cenotaph remembers this room.".to_string()],
        }],
    };

    let json = serde_json::to_string_pretty(&level).unwrap();
    std::fs::write(&temp_path, format!("{}\n", json)).unwrap();
    let loaded = LevelData::try_load(temp_path.to_str().unwrap()).unwrap();
    let _ = std::fs::remove_file(&temp_path);

    assert_eq!(loaded.name, "Save Test");
    assert_eq!(loaded.version, CURRENT_LEVEL_VERSION);
    assert_eq!(loaded.props.len(), 1);
    assert_eq!(loaded.props[0].collider_type, ColliderType::Box);
    assert_eq!(loaded.loot_tables.len(), 1);
    assert_eq!(loaded.paths.len(), 1);
    assert_eq!(loaded.events.len(), 1);
    assert_eq!(loaded.dialogues.len(), 1);
}

#[test]
fn advanced_authoring_defaults_survive_minimal_level_json() {
    let level: LevelData = serde_json::from_str(
        r#"
        {
            "name": "Minimal",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": []
        }
        "#,
    )
    .unwrap();

    assert!(level.asset_imports.is_empty());
    assert!(level.loot_tables.is_empty());
    assert!(level.paths.is_empty());
    assert!(level.events.is_empty());
    assert!(level.dialogues.is_empty());
    assert!(level.mountain_reactions.is_empty());
    assert_eq!(level.atmosphere, AtmosphereData::default());
    assert_eq!(level.base_material, SurfaceMaterialData::default());
}

#[test]
fn mountain_reaction_json_parses_defaults_and_particle_reversal() {
    let level = LevelData::from_json_str(
        r#"
        {
            "name": "Mountain Reaction Test",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": [],
            "mountain_reactions": [
                {
                    "id": "choir_reversal",
                    "particle_speed_multiplier": -1.5
                },
                {
                    "id": "ashen_squall",
                    "duration": 2.5,
                    "clear_color": [0.1, 0.2, 0.3],
                    "fog_color": [0.3, 0.2, 0.1],
                    "fog_density_multiplier": 1.5,
                    "key_light_color": [1.5, 1.0, 0.5],
                    "key_light_intensity_multiplier": 2.0,
                    "particle_color": [0.5, 0.75, 1.25],
                    "particle_speed_multiplier": 0.75,
                    "wind": [4.0, -2.0, 1.0],
                    "ambience_preset": "AshWind",
                    "ambience_volume_multiplier": 0.4
                }
            ],
            "events": [
                {
                    "id": "mountain_answers",
                    "trigger": { "kind": "Manual" },
                    "actions": [
                        {
                            "kind": "ReactMountain",
                            "reaction_id": "choir_reversal"
                        }
                    ]
                }
            ]
        }
        "#,
    )
    .unwrap();

    let reaction = &level.mountain_reactions[0];
    assert_eq!(reaction.duration, 4.0);
    assert_eq!(reaction.clear_color, None);
    assert_eq!(reaction.fog_color, None);
    assert_eq!(reaction.fog_density_multiplier, 1.0);
    assert_eq!(reaction.key_light_color, None);
    assert_eq!(reaction.key_light_intensity_multiplier, 1.0);
    assert_eq!(reaction.particle_color, None);
    assert_eq!(reaction.particle_speed_multiplier, -1.5);
    assert_eq!(reaction.wind, None);
    assert_eq!(reaction.ambience_preset, None);
    assert_eq!(reaction.ambience_volume_multiplier, 1.0);
    assert_eq!(
        level.mountain_reactions[1],
        MountainReactionData {
            id: "ashen_squall".to_string(),
            duration: 2.5,
            clear_color: Some([0.1, 0.2, 0.3]),
            fog_color: Some([0.3, 0.2, 0.1]),
            fog_density_multiplier: 1.5,
            key_light_color: Some([1.5, 1.0, 0.5]),
            key_light_intensity_multiplier: 2.0,
            particle_color: Some([0.5, 0.75, 1.25]),
            particle_speed_multiplier: 0.75,
            wind: Some([4.0, -2.0, 1.0]),
            ambience_preset: Some(AmbiencePreset::AshWind),
            ambience_volume_multiplier: 0.4,
        }
    );
    assert_eq!(
        level.events[0].actions[0].reaction_id.as_deref(),
        Some("choir_reversal")
    );
    assert_eq!(level.validate(), Ok(()));
}

#[test]
fn mountain_reaction_validation_rejects_invalid_profiles() {
    let mut level = LevelData::default_level();
    level.mountain_reactions = vec![
        MountainReactionData {
            id: "bad reaction".to_string(),
            duration: 0.0,
            clear_color: Some([-0.1, 0.0, 0.0]),
            fog_color: Some([f32::NAN, 0.0, 0.0]),
            fog_density_multiplier: -1.0,
            key_light_color: Some([3.0, 0.0, 0.0]),
            key_light_intensity_multiplier: f32::INFINITY,
            particle_color: Some([0.0, 0.0, 3.0]),
            particle_speed_multiplier: f32::NAN,
            wind: Some([0.0, f32::INFINITY, 0.0]),
            ambience_preset: None,
            ambience_volume_multiplier: -0.5,
        },
        MountainReactionData {
            id: "bad reaction".to_string(),
            ..MountainReactionData::default()
        },
    ];

    let errors = level.validation_errors();
    for field in [
        "duration",
        "clear_color",
        "fog_color",
        "fog_density_multiplier",
        "key_light_color",
        "key_light_intensity_multiplier",
        "particle_color",
        "particle_speed_multiplier",
        "wind",
        "ambience_volume_multiplier",
    ] {
        assert!(
            errors.iter().any(|error| error.contains(field)),
            "missing validation error for {field}: {errors:?}"
        );
    }
    assert!(errors
        .iter()
        .any(|error| error.contains("duplicate mountain reaction id")));
    assert!(errors
        .iter()
        .any(|error| error.contains("must use only letters")));
}

#[test]
fn react_mountain_actions_require_declared_reaction_ids() {
    let level = LevelData::from_json_str(
        r#"
        {
            "name": "Broken Mountain Reactions",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": [],
            "events": [
                {
                    "id": "missing_reaction_id",
                    "trigger": { "kind": "Manual" },
                    "actions": [{ "kind": "ReactMountain" }]
                },
                {
                    "id": "unknown_reaction_id",
                    "trigger": { "kind": "Manual" },
                    "actions": [
                        {
                            "kind": "ReactMountain",
                            "reaction_id": "undeclared_reaction"
                        }
                    ]
                }
            ]
        }
        "#,
    )
    .unwrap();

    let errors = level.validation_errors();
    assert!(errors
        .iter()
        .any(|error| error.contains("ReactMountain requires reaction_id")));
    assert!(errors.iter().any(|error| {
        error.contains("reaction_id references unknown id 'undeclared_reaction'")
    }));
}

#[test]
fn atmosphere_and_surface_material_validation_guard_runtime_budgets() {
    let atmosphere = AtmosphereData {
        particle_count: 513,
        wind: [f32::INFINITY, 0.0, 0.0],
        ..AtmosphereData::default()
    };
    let errors = atmosphere.validation_errors();
    assert!(errors.iter().any(|error| error.contains("particle_count")));
    assert!(errors.iter().any(|error| error.contains("wind")));

    let material = SurfaceMaterialData {
        texture: Some("../Cargo.toml".to_string()),
        uv_scale: 0.0,
        emissive: 5.0,
        ..SurfaceMaterialData::default()
    };
    let errors = material.validation_errors("test material");
    assert!(errors.iter().any(|error| error.contains("safe path")));
    assert!(errors.iter().any(|error| error.contains("uv_scale")));
    assert!(errors.iter().any(|error| error.contains("emissive")));
}

#[test]
fn asset_imports_can_track_source_files_outside_assets() {
    let import = AssetImportData {
        id: "source_texture".to_string(),
        asset_id: "textures/source_albedo.webp".to_string(),
        source_path: Some("Cargo.toml".to_string()),
        default_scale: [1.0, 1.0, 1.0],
        default_collider_type: ColliderType::None,
        tags: vec!["texture".to_string()],
        notes: Some("source-only authoring import".to_string()),
    };

    assert!(import.validation_errors(0).is_empty());
}

#[test]
fn validation_accepts_custom_brush_geometry_without_asset_file() {
    let level = LevelData::from_json_str(
        r#"
        {
            "name": "Brush Geometry Test",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": [
                {
                    "asset_id": "generated/brush_geometry",
                    "collider_type": "Mesh",
                    "brush_geometry": {
                        "vertices": [
                            [-1.0, 0.0, -1.0],
                            [1.0, 0.0, -1.0],
                            [1.0, 1.0, 1.0],
                            [-1.0, 0.0, 1.0]
                        ],
                        "faces": [[0, 1, 2], [0, 2, 3]]
                    }
                }
            ]
        }
        "#,
    )
    .unwrap();

    assert_eq!(level.validate(), Ok(()));
}

#[test]
fn validation_rejects_broken_custom_brush_geometry() {
    let level: LevelData = serde_json::from_str(
        r#"
        {
            "name": "Broken Brush Geometry Test",
            "base_map": "assets/Cube.obj",
            "player_spawn": [0.0, 0.0, 0.0],
            "props": [
                {
                    "asset_id": "generated/brush_geometry",
                    "collider_type": "Mesh",
                    "brush_geometry": {
                        "vertices": [
                            [0.0, 0.0, 0.0],
                            [1.0, 0.0, 0.0],
                            [2.0, 0.0, 0.0]
                        ],
                        "faces": [[0, 1, 2], [0, 1, 9]]
                    }
                }
            ]
        }
        "#,
    )
    .unwrap();
    let errors = level.validate().unwrap_err();

    assert!(errors
        .iter()
        .any(|error| error.contains("brush_geometry face 0 must not be degenerate")));
    assert!(errors
        .iter()
        .any(|error| error.contains("brush_geometry face 1 references a missing vertex")));
}

#[test]
fn validation_accepts_connected_authoring_graph() {
    let level = LevelData {
        version: CURRENT_LEVEL_VERSION,
        name: "Authoring Graph".to_string(),
        base_map: "assets/Cube.obj".to_string(),
        player_spawn: [0.0, 0.0, 0.0],
        atmosphere: AtmosphereData::default(),
        base_material: SurfaceMaterialData::default(),
        mountain_reactions: Vec::new(),
        props: vec![PropData {
            id: Some("guard_01".to_string()),
            display_name: None,
            asset_id: "Cube.obj".to_string(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            collider_type: ColliderType::Sphere,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id: None,
            resource_value: 0,
            anchor_id: None,
            enemy_type: Some("ashbound".to_string()),
            enemy_health: 10.0,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            loot_table_id: Some("guard_drops".to_string()),
            path_id: Some("guard_patrol".to_string()),
            dialogue_id: None,
            event_id: Some("guard_intro".to_string()),
        }],
        asset_imports: vec![AssetImportData {
            id: "cube_import".to_string(),
            asset_id: "Cube.obj".to_string(),
            source_path: None,
            default_scale: [1.0, 1.0, 1.0],
            default_collider_type: ColliderType::Box,
            tags: vec!["test".to_string()],
            notes: Some("fixture".to_string()),
        }],
        loot_tables: vec![LootTableData {
            id: "guard_drops".to_string(),
            rolls: 1,
            entries: vec![LootEntryData {
                weight: 2,
                item_id: None,
                resource_value: 25,
                quantity: 1,
            }],
        }],
        paths: vec![LevelPathData {
            id: "guard_patrol".to_string(),
            kind: LevelPathKind::Enemy,
            looped: true,
            speed_multiplier: 0.75,
            waypoints: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        }],
        events: vec![LevelEventData {
            id: "guard_intro".to_string(),
            once: true,
            trigger: LevelEventTriggerData {
                kind: LevelEventTriggerKind::Manual,
                position: [0.0, 0.0, 0.0],
                radius: 2.5,
                prop_id: None,
                flag_id: None,
            },
            actions: vec![LevelEventActionData {
                kind: LevelEventActionKind::StartDialogue,
                target_level_id: None,
                loot_table_id: None,
                dialogue_id: Some("guard_dialogue".to_string()),
                reaction_id: None,
                flag_id: None,
                resource_value: 0,
                spawn_position: None,
            }],
        }],
        dialogues: vec![DialogueData {
            id: "guard_dialogue".to_string(),
            speaker: "Guard".to_string(),
            lines: vec!["Stay on the path.".to_string()],
        }],
    };

    assert_eq!(level.validate(), Ok(()));
}

#[test]
fn validation_rejects_broken_authoring_references() {
    let level = LevelData {
        version: CURRENT_LEVEL_VERSION,
        name: "Broken Graph".to_string(),
        base_map: "assets/Cube.obj".to_string(),
        player_spawn: [0.0, 0.0, 0.0],
        atmosphere: AtmosphereData::default(),
        base_material: SurfaceMaterialData::default(),
        mountain_reactions: Vec::new(),
        props: vec![PropData {
            id: Some("bad prop".to_string()),
            display_name: None,
            asset_id: "Cube.obj".to_string(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            collider_type: ColliderType::None,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id: None,
            resource_value: 0,
            anchor_id: None,
            enemy_type: None,
            enemy_health: 0.0,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            loot_table_id: Some("missing_loot".to_string()),
            path_id: Some("missing_path".to_string()),
            dialogue_id: Some("missing_dialogue".to_string()),
            event_id: Some("missing_event".to_string()),
        }],
        asset_imports: Vec::new(),
        loot_tables: vec![LootTableData {
            id: "bad_table".to_string(),
            rolls: 0,
            entries: vec![LootEntryData {
                weight: 0,
                item_id: Some("ash_splinter".to_string()),
                resource_value: 5,
                quantity: 0,
            }],
        }],
        paths: vec![LevelPathData {
            id: "short_path".to_string(),
            kind: LevelPathKind::Enemy,
            looped: false,
            speed_multiplier: 0.0,
            waypoints: vec![[0.0, 0.0, 0.0]],
        }],
        events: vec![LevelEventData {
            id: "broken_event".to_string(),
            once: true,
            trigger: LevelEventTriggerData {
                kind: LevelEventTriggerKind::Interact,
                position: [0.0, 0.0, 0.0],
                radius: 0.0,
                prop_id: None,
                flag_id: None,
            },
            actions: vec![LevelEventActionData {
                kind: LevelEventActionKind::SpawnLoot,
                target_level_id: None,
                loot_table_id: Some("missing_loot".to_string()),
                dialogue_id: None,
                reaction_id: None,
                flag_id: None,
                resource_value: 0,
                spawn_position: Some([f32::NAN, 0.0, 0.0]),
            }],
        }],
        dialogues: vec![DialogueData {
            id: "empty_dialogue".to_string(),
            speaker: String::new(),
            lines: vec![String::new()],
        }],
    };

    let errors = level.validation_errors();
    assert!(errors.iter().any(|error| error.contains("bad prop")));
    assert!(errors.iter().any(|error| error.contains("missing_loot")));
    assert!(errors
        .iter()
        .any(|error| error.contains("speed_multiplier")));
    assert!(errors
        .iter()
        .any(|error| error.contains("interact triggers require prop_id")));
    assert!(errors.iter().any(|error| error.contains("spawn_position")));
    assert!(errors.iter().any(|error| error.contains("speaker")));
}
