//! Level loading, preparation, and save-state restoration.
//!
//! These `impl EngineState` methods handle disk I/O, validation, and
//! construction of runtime state from authored level data and save files.

use std::collections::{HashSet, VecDeque};
use std::path::Path;

use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::core::engine::state::EngineState;
use crate::core::engine::loader::{load_prop_assets, load_textures_from_disk};
use crate::core::engine::validation::{tuning_validation_errors, validate_model_geometry};
use crate::data::config::gameplay::{GameConfig, PhysicsConfig};
use crate::data::enemy::EnemyRegistry;
use crate::data::relic::RelicRegistry;
use crate::data::world::level::{
    validate_level_id, LevelData, LevelEventActionKind, LevelEventData, LevelEventTriggerKind,
    PropData, BASE_MAP_Y_OFFSET,
};
use crate::game::enemy::EnemyRuntimeState;
use crate::game::progression::RunProgress;
use crate::game::save::SaveData;
use crate::systems::render::assets::AssetManager;
use crate::systems::render::instance::InstanceRaw;
use crate::systems::render::mesh::{try_load_model, ModelData, RenderMeshPart, Vertex};
use crate::systems::render::texture::TextureManager;

pub(super) struct PreparedLevel {
    pub(super) name: String,
    pub(super) path: String,
    pub(super) data: LevelData,
    pub(super) model: ModelData,
}

pub(super) struct MapRenderResources {
    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) parts: Vec<crate::systems::render::assets::RenderAssetMeshPart>,
    pub(super) instance_buffer: wgpu::Buffer,
    pub(super) texture_override: Option<String>,
}

impl EngineState {
    /// Reloads designer-owned runtime data as one transaction. Nothing live is
    /// changed unless config, registries, level data, and map geometry all pass.
    pub fn reload_runtime_content(&mut self) -> Result<(), String> {
        let config = GameConfig::try_load()
            .map_err(|error| format!("game configuration could not be reloaded: {}", error))?;
        let tuning_errors = tuning_validation_errors(&config);
        if !tuning_errors.is_empty() {
            return Err(format!(
                "gameplay tuning failed validation: {}",
                tuning_errors.join("; ")
            ));
        }

        let enemy_registry = EnemyRegistry::try_load_dir("data/enemies")
            .map_err(|error| format!("enemy definitions could not be reloaded: {}", error))?;
        let relic_registry = RelicRegistry::try_load_dir("data/relics")
            .map_err(|error| format!("relic definitions could not be reloaded: {}", error))?;
        let prepared = Self::prepare_level(&self.level_name, &enemy_registry, &relic_registry)?;

        let owned_relic_ids = self.equipped_relic.owned_ids();
        let equipped_relic_id = self.equipped_relic.equipped_id().map(ToOwned::to_owned);
        let missing_owned_relics = owned_relic_ids
            .iter()
            .filter(|id| relic_registry.get(id).is_none())
            .cloned()
            .collect::<Vec<_>>();
        if !missing_owned_relics.is_empty() {
            return Err(format!(
                "relic reload would remove owned relic(s): {}",
                missing_owned_relics.join(", ")
            ));
        }

        let mut assets = AssetManager::new();
        let asset_count =
            load_prop_assets(&self.device, &mut assets).into_result("model asset reload")?;
        let mut texture_manager =
            TextureManager::new(&self.device, &self.queue, &self.texture_bind_group_layout);
        let texture_count = load_textures_from_disk(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &mut texture_manager,
        )
        .into_result("texture reload")?;

        self.player.reconfigure(&config.player);
        self.camera_controller
            .set_sensitivity(config.camera.sensitivity);
        self.camera.zfar = config.world.draw_distance;
        self.config_data = config;
        self.hud.set_ui_config(&self.config_data.ui);
        self.enemy_registry = enemy_registry;
        self.relic_registry = relic_registry;
        self.assets = assets;
        self.texture_manager = texture_manager;
        self.equipped_relic.restore_from_ids(
            &owned_relic_ids,
            equipped_relic_id.as_deref(),
            &self.relic_registry,
        );
        self.commit_prepared_level(prepared);
        println!(
            "[RELOAD] Runtime data applied: {} models, {} textures, {} enemies, {} relics",
            asset_count,
            texture_count,
            self.enemy_registry.len(),
            self.relic_registry.len()
        );
        Ok(())
    }

    /// Fully prepares a new level before replacing any live runtime state.
    pub fn load_level(&mut self, new_level_name: &str) -> Result<(), String> {
        let prepared =
            Self::prepare_level(new_level_name, &self.enemy_registry, &self.relic_registry)?;
        self.commit_prepared_level(prepared);
        Ok(())
    }

    pub(super) fn commit_prepared_level(&mut self, prepared: PreparedLevel) {
        let PreparedLevel {
            name,
            path,
            data: level_data,
            model,
        } = prepared;
        let ModelData {
            vertices: map_vertices,
            parts: map_mesh_parts,
            physics_vertices,
            physics_triangles,
        } = model;
        let map_resources = Self::build_map_resources(
            &self.device,
            &map_vertices,
            map_mesh_parts,
            &level_data.base_material,
        );
        let physics = Self::build_physics_for_level(
            &level_data,
            &self.config_data.physics,
            physics_vertices,
            physics_triangles,
        );
        let enemy_runtime = Self::enemy_runtime_for_level(&level_data);
        let level_event_fired = Self::level_event_runtime_for_level(&level_data);

        self.physics = physics;
        self.camera.position = Self::camera_position_for_spawn(level_data.player_spawn);
        self.map_vertex_buffer = map_resources.vertex_buffer;
        self.map_parts = map_resources.parts;
        self.map_instance_buffer = map_resources.instance_buffer;
        self.map_texture_override = map_resources.texture_override;
        self.level_data = level_data;
        self.runtime_atmosphere = self.level_data.atmosphere.clone();
        self.level_name = name;
        self.enemy_runtime = enemy_runtime;
        self.level_event_fired = level_event_fired;
        self.level_flags.clear();
        self.queued_manual_level_events.clear();
        self.active_dialogue = None;
        self.active_anchor_rite = None;
        self.mountain_reaction = None;
        self.queued_mountain_reactions.clear();
        self.removed_prop_ids.clear();
        self.failed_transition = None;
        self.particles
            .configure(&self.queue, &self.runtime_atmosphere, self.camera.position);
        if let Some(audio) = self.audio.as_mut() {
            audio.set_ambience(
                self.runtime_atmosphere.ambience_preset,
                self.runtime_atmosphere.ambience_volume,
            );
        }

        self.action_cooldown = 0.0;
        self.progress.clear_anchor();
        self.player
            .reset_for_level_transition(&self.config_data.player);

        self.sync_instances();
        println!(
            "[LEVEL] Loaded '{}' from {} ({} props)",
            self.level_name,
            path,
            self.level_data.props.len()
        );
    }

    pub fn update_lighting(&mut self) {
        let light_pos = [
            self.camera.position.x + 0.5,
            self.camera.position.y + self.config_data.lighting.sun_position_offset,
            self.camera.position.z + 0.3,
        ];
        self.lighting.update_light(
            &self.queue,
            light_pos,
            self.runtime_atmosphere.key_light_color,
            self.runtime_atmosphere.key_light_intensity,
        );

        self.lighting.update_fog(
            &self.queue,
            self.runtime_atmosphere.fog_density,
            self.runtime_atmosphere.fog_color,
        );
    }

    pub(super) fn build_map_resources(
        device: &wgpu::Device,
        vertices: &[Vertex],
        mesh_parts: Vec<RenderMeshPart>,
        material: &crate::data::world::level::SurfaceMaterialData,
    ) -> MapRenderResources {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let parts = mesh_parts
            .into_iter()
            .map(|part| {
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Map Index Buffer"),
                    contents: bytemuck::cast_slice(&part.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                crate::systems::render::assets::RenderAssetMeshPart {
                    index_buffer,
                    num_indices: part.indices.len() as u32,
                    texture_name: part.texture_name,
                }
            })
            .collect();

        let map_instance = [InstanceRaw {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, BASE_MAP_Y_OFFSET, 0.0, 1.0],
            ],
            tint: [
                0.72 * material.tint[0],
                0.74 * material.tint[1],
                0.78 * material.tint[2],
                1.0,
            ],
            material: [material.uv_scale, material.emissive, 0.0, 0.0],
        }];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map Instance Buffer"),
            contents: bytemuck::cast_slice(&map_instance),
            usage: wgpu::BufferUsages::VERTEX,
        });

        MapRenderResources {
            vertex_buffer,
            parts,
            instance_buffer,
            texture_override: material.texture.clone(),
        }
    }

    pub(super) fn build_physics_for_level(
        level_data: &LevelData,
        config: &PhysicsConfig,
        physics_vertices: Vec<Vec3>,
        physics_triangles: Vec<[u32; 3]>,
    ) -> crate::systems::physics::engine::PhysicsEngine {
        let mut physics = crate::systems::physics::engine::PhysicsEngine::new(
            level_data.player_spawn,
            physics_vertices,
            physics_triangles,
            config,
        );

        for prop in &level_data.props {
            if let Some((prop_points, prop_indices)) = Self::brush_physics_mesh(prop) {
                physics.add_prop(prop, &prop_points, &prop_indices);
                continue;
            }

            let asset_path = format!("assets/{}", prop.asset_id);
            match try_load_model(&asset_path) {
                Ok(model) => {
                    physics.add_prop(prop, &model.physics_vertices, &model.physics_triangles);
                }
                Err(error) => {
                    eprintln!(
                        "[ERROR] Failed to load prop model '{}': {}",
                        asset_path, error
                    );
                    physics.add_prop(prop, &[], &[]);
                }
            }
        }

        physics
    }

    fn brush_physics_mesh(prop: &PropData) -> Option<(Vec<Vec3>, Vec<[u32; 3]>)> {
        let geometry = prop.brush_geometry.as_ref()?;
        let points = geometry
            .vertices
            .iter()
            .map(|vertex| Vec3::from_array(*vertex))
            .collect();
        Some((points, geometry.faces.clone()))
    }

    pub(super) fn camera_for_spawn(
        spawn: [f32; 3],
        width: u32,
        height: u32,
        draw_distance: f32,
    ) -> crate::systems::render::camera::Camera {
        crate::systems::render::camera::Camera {
            position: Self::camera_position_for_spawn(spawn),
            yaw: -1.5,
            pitch: 0.0,
            visual_yaw_offset: 0.0,
            visual_pitch_offset: 0.0,
            aspect: width as f32 / height as f32,
            fovy: crate::systems::render::camera::BASE_FOVY,
            znear: 0.1,
            zfar: draw_distance,
        }
    }

    pub(super) fn camera_position_for_spawn(spawn: [f32; 3]) -> Vec3 {
        Vec3::new(spawn[0], spawn[1] + BASE_MAP_Y_OFFSET + 1.0, spawn[2])
    }

    pub(super) fn create_depth_resources(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        (depth_texture, depth_view)
    }

    pub(super) fn log_map_model(vertices: &[Vertex], part_count: usize) {
        println!(
            "[DEBUG] Render: {} map vertices, {} mesh parts",
            vertices.len(),
            part_count
        );
    }

    pub(super) fn prepare_level(
        level_name: &str,
        enemy_registry: &EnemyRegistry,
        relic_registry: &RelicRegistry,
    ) -> Result<PreparedLevel, String> {
        validate_level_id(level_name)?;
        let path = format!("levels/{}.json", level_name);
        let mut data = LevelData::try_load(&path)?;
        Self::apply_enemy_definitions(&mut data, enemy_registry, &path)?;

        let mut validation_errors = data.validation_errors();
        validation_errors.extend(Self::relic_reference_errors(&data, relic_registry));
        if !validation_errors.is_empty() {
            return Err(format!(
                "level '{}' failed validation: {}",
                path,
                validation_errors.join("; ")
            ));
        }

        let model = try_load_model(&data.base_map).map_err(|error| {
            format!(
                "failed to load base map '{}' for level '{}': {}",
                data.base_map, level_name, error
            )
        })?;
        let model_errors = validate_model_geometry(&model);
        if !model_errors.is_empty() {
            return Err(format!(
                "base map '{}' failed geometry validation: {}",
                data.base_map,
                model_errors.join("; ")
            ));
        }

        Ok(PreparedLevel {
            name: level_name.to_string(),
            path,
            data,
            model,
        })
    }

    fn apply_enemy_definitions(
        level_data: &mut LevelData,
        enemy_registry: &EnemyRegistry,
        level_path: &str,
    ) -> Result<(), String> {
        for (index, prop) in level_data.props.iter_mut().enumerate() {
            let Some(enemy_type) = prop.enemy_type.as_deref() else {
                continue;
            };

            let Some(enemy) = enemy_registry.get(enemy_type) else {
                return Err(format!(
                    "level '{}' prop {} references unknown enemy_type '{}'",
                    level_path, index, enemy_type
                ));
            };

            prop.asset_id = enemy.model_asset.clone();
            prop.collider_type = enemy.collider_type;
            if prop.enemy_health <= 0.0 {
                prop.enemy_health = enemy.health;
            }
        }
        Ok(())
    }

    fn relic_reference_errors(
        level_data: &LevelData,
        relic_registry: &RelicRegistry,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        for (index, prop) in level_data.props.iter().enumerate() {
            if let Some(item_id) = prop.item_id.as_deref() {
                if relic_registry.get(item_id).is_none() {
                    errors.push(format!(
                        "prop {} references unknown item_id '{}'",
                        index, item_id
                    ));
                }
            }
        }
        for (table_index, table) in level_data.loot_tables.iter().enumerate() {
            for (entry_index, entry) in table.entries.iter().enumerate() {
                if let Some(item_id) = entry.item_id.as_deref() {
                    if relic_registry.get(item_id).is_none() {
                        errors.push(format!(
                            "loot table {} entry {} references unknown item_id '{}'",
                            table_index, entry_index, item_id
                        ));
                    }
                }
            }
        }
        errors
    }

    pub(super) fn enemy_runtime_for_level(level_data: &LevelData) -> Vec<EnemyRuntimeState> {
        level_data
            .props
            .iter()
            .map(|prop| {
                EnemyRuntimeState::for_max_health(
                    prop.enemy_type.as_ref().map_or(0.0, |_| prop.enemy_health),
                )
            })
            .collect()
    }

    pub(super) fn level_event_runtime_for_level(level_data: &LevelData) -> Vec<bool> {
        vec![false; level_data.events.len()]
    }

    pub(super) fn manual_level_event_status(
        &self,
        event_id: &str,
    ) -> super::state::ManualLevelEventStatus {
        Self::manual_level_event_status_for(
            &self.level_data.events,
            &self.level_event_fired,
            &self.level_flags,
            event_id,
        )
    }

    pub(super) fn manual_level_event_status_for(
        events: &[LevelEventData],
        fired: &[bool],
        flags: &HashSet<String>,
        event_id: &str,
    ) -> super::state::ManualLevelEventStatus {
        use super::state::ManualLevelEventStatus;
        let Some((index, event)) = events
            .iter()
            .enumerate()
            .find(|(_, event)| event.id == event_id)
        else {
            return ManualLevelEventStatus::MissingEvent;
        };
        if event.trigger.kind != LevelEventTriggerKind::Manual {
            return ManualLevelEventStatus::WrongTrigger(event.trigger.kind);
        }
        if let Some(flag_id) = event.trigger.flag_id.as_deref() {
            if !flags.contains(flag_id) {
                return ManualLevelEventStatus::MissingFlag(flag_id.to_string());
            }
        }
        if event.once && fired.get(index).copied().unwrap_or(false) {
            return ManualLevelEventStatus::AlreadyFired;
        }
        ManualLevelEventStatus::Ready
    }

    pub(super) fn level_event_runtime_for_saved_level(
        level_data: &LevelData,
        save: Option<&SaveData>,
    ) -> (Vec<bool>, bool) {
        let Some(save) = save else {
            return (Self::level_event_runtime_for_level(level_data), false);
        };

        let fired: HashSet<&str> = save.fired_level_events.iter().map(String::as_str).collect();
        let runtime = level_data
            .events
            .iter()
            .map(|event| event.once && fired.contains(event.id.as_str()))
            .collect::<Vec<_>>();
        let restored_count = runtime.iter().filter(|fired| **fired).count();
        (runtime, restored_count != save.fired_level_events.len())
    }

    pub(super) fn level_flags_for_saved_level(
        level_data: &LevelData,
        save: Option<&SaveData>,
    ) -> (HashSet<String>, bool) {
        let Some(save) = save else {
            return (HashSet::new(), false);
        };
        let known_ids = level_data
            .events
            .iter()
            .flat_map(|event| {
                event
                    .trigger
                    .flag_id
                    .iter()
                    .chain(event.actions.iter().filter_map(|action| {
                        (action.kind == LevelEventActionKind::SetFlag)
                            .then_some(action.flag_id.as_ref())
                            .flatten()
                    }))
            })
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let flags = save
            .level_flags
            .iter()
            .filter(|flag_id| known_ids.contains(flag_id.as_str()))
            .cloned()
            .collect::<HashSet<_>>();
        let cleanup_needed = flags.len() != save.level_flags.len();
        (flags, cleanup_needed)
    }

    pub(super) fn progress_for_saved_level(
        level_data: &LevelData,
        save: &SaveData,
    ) -> (RunProgress, bool) {
        let mut progress = save.to_progress();
        match (
            save.active_anchor_id.as_deref(),
            save.respawn_position.as_ref(),
        ) {
            (None, None) => (progress, false),
            (Some(anchor_id), Some(saved_position)) => {
                let current_position = level_data
                    .props
                    .iter()
                    .find(|prop| prop.anchor_id.as_deref() == Some(anchor_id))
                    .map(|prop| prop.position);
                if let Some(current_position) = current_position {
                    let cleanup_needed = current_position != *saved_position;
                    progress.respawn_position = Some(current_position);
                    (progress, cleanup_needed)
                } else {
                    eprintln!("[SAVE] Ignoring obsolete active anchor '{}'", anchor_id);
                    progress.clear_anchor();
                    (progress, true)
                }
            }
            _ => {
                eprintln!("[SAVE] Ignoring incomplete active anchor state");
                progress.clear_anchor();
                (progress, true)
            }
        }
    }

    pub(super) fn mountain_reactions_for_saved_level(
        level_data: &LevelData,
        save: Option<&SaveData>,
    ) -> (VecDeque<String>, bool) {
        let Some(save) = save else {
            return (VecDeque::new(), false);
        };
        let known_ids = level_data
            .mountain_reactions
            .iter()
            .map(|reaction| reaction.id.as_str())
            .collect::<HashSet<_>>();
        let queue = save
            .pending_mountain_reactions
            .iter()
            .filter_map(|reaction_id| {
                if known_ids.contains(reaction_id.as_str()) {
                    Some(reaction_id.clone())
                } else {
                    eprintln!(
                        "[SAVE] Ignoring obsolete mountain reaction '{}'",
                        reaction_id
                    );
                    None
                }
            })
            .collect::<VecDeque<_>>();
        let cleanup_needed = queue.len() != save.pending_mountain_reactions.len();
        (queue, cleanup_needed)
    }

    pub(super) fn apply_saved_world_state(
        level_data: &mut LevelData,
        save: &SaveData,
        relic_registry: &RelicRegistry,
    ) -> (HashSet<String>, bool) {
        let removable_authored_ids = level_data
            .props
            .iter()
            .filter(|prop| {
                prop.enemy_type.is_some() || prop.item_id.is_some() || prop.resource_value > 0
            })
            .filter_map(|prop| prop.id.as_deref())
            .collect::<HashSet<_>>();
        let removed_prop_ids = save
            .removed_prop_ids
            .iter()
            .filter(|prop_id| removable_authored_ids.contains(prop_id.as_str()))
            .cloned()
            .collect::<HashSet<_>>();
        let mut cleanup_needed = removed_prop_ids.len() != save.removed_prop_ids.len();
        level_data.props.retain(|prop| {
            prop.id
                .as_deref()
                .is_none_or(|prop_id| !removed_prop_ids.contains(prop_id))
        });

        for loot in &save.runtime_loot {
            let already_present = level_data
                .props
                .iter()
                .any(|prop| prop.id.as_deref() == Some(loot.id.as_str()));
            if already_present {
                cleanup_needed = true;
                continue;
            }

            let mut prop = loot.to_prop();
            if let Some(item_id) = loot.item_id.as_deref() {
                let Some(relic) = relic_registry.get(item_id) else {
                    eprintln!(
                        "[SAVE] Ignoring runtime loot '{}' for removed relic '{}'",
                        loot.id, item_id
                    );
                    cleanup_needed = true;
                    continue;
                };
                cleanup_needed |= prop.asset_id != relic.pickup_asset;
                prop.asset_id = relic.pickup_asset.clone();
            } else if !Path::new("assets").join(&loot.asset_id).is_file() {
                eprintln!(
                    "[SAVE] Ignoring runtime resource '{}' with missing asset 'assets/{}'",
                    loot.id, loot.asset_id
                );
                cleanup_needed = true;
                continue;
            }
            level_data.props.push(prop);
        }
        (removed_prop_ids, cleanup_needed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::enemy::EnemyDefinition;
    use crate::data::world::level::{
        ColliderType, LevelEventData, LevelEventTriggerData, LevelEventTriggerKind, PropData,
    };
    use crate::game::cycle::CycleState;
    use crate::game::relic::EquippedRelic;

    #[test]
    fn default_runtime_content_prepares_as_one_valid_bundle() {
        let enemies = EnemyRegistry::try_load_dir("data/enemies").unwrap();
        let relics = RelicRegistry::try_load_dir("data/relics").unwrap();

        let prepared = EngineState::prepare_level("movement_test", &enemies, &relics).unwrap();

        assert_eq!(prepared.name, "movement_test");
        assert!(!prepared.data.props.is_empty());
        assert!(!prepared.model.vertices.is_empty());
    }

    #[test]
    fn first_ascent_prepares_with_materialized_enemies() {
        let enemies = EnemyRegistry::try_load_dir("data/enemies").unwrap();
        let relics = RelicRegistry::try_load_dir("data/relics").unwrap();

        let prepared = EngineState::prepare_level("ashwalk_01", &enemies, &relics).unwrap();

        assert!(prepared.data.props.len() <= 14);
        assert!(!prepared.model.vertices.is_empty());
        assert!(prepared
            .data
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some())
            .all(|prop| prop.enemy_health > 0.0));
        let ashwarden = prepared
            .data
            .props
            .iter()
            .find(|prop| prop.id.as_deref() == Some("ashwarden_elite"))
            .unwrap();
        assert_eq!(ashwarden.enemy_type.as_deref(), Some("ashwarden"));
        assert_eq!(ashwarden.enemy_health, 220.0);
        assert_eq!(ashwarden.event_id.as_deref(), Some("ashwarden_fall"));
        let activation_range = enemies.get("ashwarden").unwrap().activation_range;
        let warden_position = Vec3::from_array(ashwarden.position);
        let final_trail_position = prepared
            .data
            .props
            .iter()
            .find(|prop| prop.id.as_deref() == Some("trail_resource_03"))
            .map(|prop| Vec3::from_array(prop.position))
            .unwrap();
        let anchor_position = prepared
            .data
            .props
            .iter()
            .find(|prop| prop.anchor_id.as_deref() == Some("ashwalk_summit"))
            .map(|prop| Vec3::from_array(prop.position))
            .unwrap();
        assert!(
            Vec3::from_array(prepared.data.player_spawn).distance(warden_position)
                > activation_range
        );
        assert!(final_trail_position.distance(warden_position) < activation_range);
        assert!(anchor_position.distance(warden_position) < activation_range);
        assert_eq!(prepared.data.mountain_reactions.len(), 2);
        assert!(relics.get("debt_of_the_last_keeper").is_some());
    }

    #[test]
    fn enemy_definitions_materialize_level_props() {
        let enemy_registry = EnemyRegistry::from_definitions(vec![EnemyDefinition {
            id: "burdened".to_string(),
            display_name: "Burdened".to_string(),
            role: "tank".to_string(),
            behavior_tag: "slow_chase_melee".to_string(),
            model_asset: "enemies/burdened.obj".to_string(),
            collider_type: ColliderType::Sphere,
            visual_tell: "wide tank silhouette".to_string(),
            health: 120.0,
            damage: 18.0,
            move_speed: 1.4,
            activation_range: 14.0,
            attack_range: 1.8,
            attack_windup: 0.75,
            attack_cooldown: 1.8,
        }])
        .unwrap();
        let mut level = LevelData {
            version: crate::data::world::level::CURRENT_LEVEL_VERSION,
            name: "Materialize Test".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            atmosphere: Default::default(),
            base_material: Default::default(),
            mountain_reactions: Vec::new(),
            props: vec![PropData {
                id: None,
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
                enemy_type: Some("Burdened".to_string()),
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
            loot_tables: Vec::new(),
            paths: Vec::new(),
            events: Vec::new(),
            dialogues: Vec::new(),
        };
        let mut authored_elite = level.props[0].clone();
        authored_elite.id = Some("named_elite".to_string());
        authored_elite.enemy_health = 220.0;
        level.props.push(authored_elite);

        EngineState::apply_enemy_definitions(&mut level, &enemy_registry, "test").unwrap();

        let prop = &level.props[0];
        assert_eq!(prop.asset_id, "enemies/burdened.obj");
        assert_eq!(prop.collider_type, ColliderType::Sphere);
        assert_eq!(prop.enemy_health, 120.0);
        assert_eq!(level.props[1].enemy_health, 220.0);
    }

    #[test]
    fn enemy_runtime_state_matches_prop_slots() {
        let level = LevelData {
            version: crate::data::world::level::CURRENT_LEVEL_VERSION,
            name: "Cooldown Test".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            atmosphere: Default::default(),
            base_material: Default::default(),
            mountain_reactions: Vec::new(),
            props: vec![
                PropData {
                    id: None,
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
                    loot_table_id: None,
                    path_id: None,
                    dialogue_id: None,
                    event_id: None,
                },
                PropData {
                    id: None,
                    display_name: None,
                    asset_id: "Cube.obj".to_string(),
                    position: [1.0, 0.0, 0.0],
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
                    enemy_type: Some("Ashbound".to_string()),
                    enemy_health: 40.0,
                    light_color: None,
                    light_intensity: 0.0,
                    ambient_sound_id: None,
                    trigger_level_id: None,
                    loot_table_id: None,
                    path_id: None,
                    dialogue_id: None,
                    event_id: None,
                },
            ],
            asset_imports: Vec::new(),
            loot_tables: Vec::new(),
            paths: Vec::new(),
            events: Vec::new(),
            dialogues: Vec::new(),
        };

        assert_eq!(
            EngineState::enemy_runtime_for_level(&level),
            vec![
                EnemyRuntimeState::default(),
                EnemyRuntimeState::for_max_health(40.0)
            ]
        );
    }

    #[test]
    fn saved_level_event_state_restores_against_current_level_ids() {
        let mut level = LevelData::default_level();
        let mut boss_door = test_level_event("boss_door");
        boss_door.trigger.flag_id = Some("gate_open".to_string());
        let mut repeatable_rite = test_level_event("repeatable_rite");
        repeatable_rite.once = false;
        repeatable_rite.trigger.kind = LevelEventTriggerKind::Interact;
        level.events = vec![
            test_level_event("intro"),
            test_level_event("gate_open_reward"),
            boss_door,
            repeatable_rite,
        ];
        let save = SaveData {
            version: crate::game::save::SAVE_VERSION,
            level_name: "foundation_test".to_string(),
            cycle_number: 1,
            unsecured_resource: 0,
            banked_resource: 0,
            active_anchor_id: None,
            respawn_position: None,
            relic_inventory: Vec::new(),
            equipped_relic_id: None,
            fired_level_events: vec![
                "missing_old_event".to_string(),
                "gate_open_reward".to_string(),
                "repeatable_rite".to_string(),
            ],
            level_flags: vec!["gate_open".to_string(), "obsolete_flag".to_string()],
            removed_prop_ids: Vec::new(),
            runtime_loot: Vec::new(),
            pending_mountain_reactions: Vec::new(),
        };

        let (fired, event_cleanup_needed) =
            EngineState::level_event_runtime_for_saved_level(&level, Some(&save));
        let (flags, flag_cleanup_needed) =
            EngineState::level_flags_for_saved_level(&level, Some(&save));

        assert_eq!(fired, vec![false, true, false, false]);
        assert!(event_cleanup_needed);
        assert!(flags.contains("gate_open"));
        assert_eq!(flags.len(), 1);
        assert!(flag_cleanup_needed);
    }

    #[test]
    fn manual_event_readiness_distinguishes_requirements_from_consumed_events() {
        let mut event = test_level_event("anchor_claim");
        event.trigger.kind = LevelEventTriggerKind::Manual;
        event.trigger.flag_id = Some("keeper_fallen".to_string());
        let events = vec![event];
        let mut flags = HashSet::new();
        use super::super::state::ManualLevelEventStatus;

        assert_eq!(
            EngineState::manual_level_event_status_for(&events, &[false], &flags, "anchor_claim"),
            ManualLevelEventStatus::MissingFlag("keeper_fallen".to_string())
        );

        flags.insert("keeper_fallen".to_string());
        assert_eq!(
            EngineState::manual_level_event_status_for(&events, &[false], &flags, "anchor_claim"),
            ManualLevelEventStatus::Ready
        );
        assert_eq!(
            EngineState::manual_level_event_status_for(&events, &[true], &flags, "anchor_claim"),
            ManualLevelEventStatus::AlreadyFired
        );
    }

    #[test]
    fn saved_world_state_removes_authored_props_and_restores_loose_loot() {
        let relics = RelicRegistry::try_load_dir("data/relics").unwrap();
        let mut level = LevelData::default_level();
        let mut defeated: PropData =
            serde_json::from_str(r#"{ "id": "warden", "asset_id": "Cube.obj" }"#).unwrap();
        defeated.enemy_type = Some("ashbound".to_string());
        defeated.enemy_health = 50.0;
        let remaining: PropData = serde_json::from_str(
            r#"{ "id": "remaining_pickup", "asset_id": "Cube.obj", "resource_value": 5 }"#,
        )
        .unwrap();
        level.props = vec![defeated, remaining];

        let mut save = SaveData::from_runtime(
            "foundation_test",
            &RunProgress::new(),
            &EquippedRelic::new(),
            &CycleState::new(1),
        );
        save.removed_prop_ids = vec!["warden".to_string()];
        save.runtime_loot = vec![
            crate::game::save::SavedRuntimeLoot {
                id: "runtime_loot_abcd_0".to_string(),
                asset_id: "pickups/resource_shard.obj".to_string(),
                position: [3.0, 4.0, 5.0],
                scale: [0.35, 0.35, 0.35],
                item_id: None,
                resource_value: 10,
            },
            crate::game::save::SavedRuntimeLoot {
                id: "runtime_loot_abcd_1".to_string(),
                asset_id: "obsolete/relic.obj".to_string(),
                position: [4.0, 4.0, 5.0],
                scale: [0.35, 0.35, 0.35],
                item_id: Some("debt_of_the_last_keeper".to_string()),
                resource_value: 0,
            },
        ];

        let (removed_ids, cleanup_needed) =
            EngineState::apply_saved_world_state(&mut level, &save, &relics);

        assert_eq!(removed_ids, HashSet::from(["warden".to_string()]));
        assert!(cleanup_needed);
        assert!(level
            .props
            .iter()
            .all(|prop| prop.id.as_deref() != Some("warden")));
        assert!(level
            .props
            .iter()
            .any(|prop| prop.id.as_deref() == Some("remaining_pickup")));
        assert!(level
            .props
            .iter()
            .any(|prop| prop.id.as_deref() == Some("runtime_loot_abcd_0")));
        let restored_relic = level
            .props
            .iter()
            .find(|prop| prop.id.as_deref() == Some("runtime_loot_abcd_1"))
            .unwrap();
        assert_eq!(restored_relic.asset_id, "pickups/relic_chain_sigil.obj");
    }

    #[test]
    fn saved_mountain_queue_ignores_profiles_removed_by_content_updates() {
        let level = LevelData::try_load("levels/ashwalk_01.json").unwrap();
        let mut save = SaveData::from_runtime(
            "ashwalk_01",
            &RunProgress::new(),
            &EquippedRelic::new(),
            &CycleState::new(1),
        );
        save.pending_mountain_reactions = vec![
            "obsolete_answer".to_string(),
            "first_claim_bound".to_string(),
        ];

        assert_eq!(
            EngineState::mountain_reactions_for_saved_level(&level, Some(&save)),
            (VecDeque::from(["first_claim_bound".to_string()]), true)
        );
    }

    #[test]
    fn saved_anchor_uses_current_authored_position_and_rejects_removed_anchors() {
        let mut level = LevelData::default_level();
        let mut anchor: PropData = serde_json::from_str(
            r#"{ "id": "first_anchor_prop", "asset_id": "Cube.obj", "position": [5.0, 6.0, 7.0], "anchor_id": "first_anchor" }"#,
        )
        .unwrap();
        anchor.display_name = Some("The First Anchor".to_string());
        level.props = vec![anchor];
        let mut save = SaveData::from_runtime(
            "foundation_test",
            &RunProgress::new(),
            &EquippedRelic::new(),
            &CycleState::new(1),
        );
        save.active_anchor_id = Some("first_anchor".to_string());
        save.respawn_position = Some([1.0, 2.0, 3.0]);

        let (progress, cleanup_needed) = EngineState::progress_for_saved_level(&level, &save);

        assert_eq!(progress.active_anchor_id.as_deref(), Some("first_anchor"));
        assert_eq!(progress.respawn_position, Some([5.0, 6.0, 7.0]));
        assert!(cleanup_needed);

        level.props.clear();
        let (progress, cleanup_needed) = EngineState::progress_for_saved_level(&level, &save);
        assert!(progress.active_anchor_id.is_none());
        assert!(progress.respawn_position.is_none());
        assert!(cleanup_needed);
    }

    #[test]
    fn saved_removals_cannot_delete_non_consumable_world_props() {
        let relics = RelicRegistry::try_load_dir("data/relics").unwrap();
        let mut level = LevelData::default_level();
        let anchor: PropData = serde_json::from_str(
            r#"{ "id": "anchor_prop", "asset_id": "Cube.obj", "anchor_id": "anchor" }"#,
        )
        .unwrap();
        level.props = vec![anchor];
        let mut save = SaveData::from_runtime(
            "foundation_test",
            &RunProgress::new(),
            &EquippedRelic::new(),
            &CycleState::new(1),
        );
        save.removed_prop_ids = vec!["anchor_prop".to_string()];

        let (removed_ids, cleanup_needed) =
            EngineState::apply_saved_world_state(&mut level, &save, &relics);

        assert!(removed_ids.is_empty());
        assert!(cleanup_needed);
        assert!(level
            .props
            .iter()
            .any(|prop| prop.id.as_deref() == Some("anchor_prop")));
    }

    fn test_level_event(id: &str) -> LevelEventData {
        LevelEventData {
            id: id.to_string(),
            once: true,
            trigger: LevelEventTriggerData {
                kind: LevelEventTriggerKind::OnEnter,
                position: [0.0, 0.0, 0.0],
                radius: 2.5,
                prop_id: None,
                flag_id: None,
            },
            actions: Vec::new(),
        }
    }
}
