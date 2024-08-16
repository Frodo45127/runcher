//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;
use getset::Getters;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use rpfm_lib::schema::Schema;
use rpfm_lib::files::{Container, ContainerPath, db::DB, DecodeableExtraData, EncodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::GameInfo;

use crate::app_ui::AppUI;
use crate::games::{EMPTY_CA_VP8, rename_file_name_to_low_priority};

const SCRIPT_DEBUG_ACTIVATOR_PATH: &str = "script/enable_console_logging";

const INTRO_MOVIE_PATHS_BY_GAME: [&str; 19] = [
    "movies/epilepsy_warning/epilepsy_warning_br.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_cn.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_cz.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_de.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_en.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_es.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_fr.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_it.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_kr.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_pl.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_ru.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_tr.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_zh.ca_vp8",
    "movies/gam_int.ca_vp8",
    "movies/startup_movie_01.ca_vp8",
    "movies/startup_movie_02.ca_vp8",
    "movies/startup_movie_03.ca_vp8",
    "movies/startup_movie_04.ca_vp8",
    "movies/startup_movie_05.ca_vp8",
];

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Default, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct UniversalRebalancerLandUnit {
    key: String,
    category: String,

    // Campaign movement.
    campaign_action_points: (i32, i32),

    // General stats.
    morale: (i32, i32),
    melee_attack: (i32, i32),
    melee_defence: (i32, i32),
    charge_bonus: (i32, i32),
    bonus_hit_points: (i32, i32),

    // Ranged stats.
    primary_ammo: (i32, i32),
    secondary_ammo: (i32, i32),
    accuracy: (i32, i32),
    reload: (i32, i32),

    // Damage modifiers.
    damage_mod_flame: (i32, i32),
    damage_mod_magic: (i32, i32),
    damage_mod_physical: (i32, i32),
    damage_mod_missile: (i32, i32),
    damage_mod_all: (i32, i32),

    // Effect modifiers.
    healing_power: (f32, f32),
    spell_mastery: (f32, f32),

    // Visibility modifiers.
    visibility_spotting_range_min: (f32, f32),
    visibility_spotting_range_max: (f32, f32),
    spot_dist_tree: (i32, i32),
    spot_dist_scrub: (i32, i32),
    hiding_scalar: (f32, f32),

    // Unit size modifiers.
    //num_mounts: (i32, i32),
    //num_engines: (i32, i32),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//


pub unsafe fn prepare_trait_limit_removal(game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema) -> Result<()> {
    let mut campaign_variables = vanilla_pack.files_by_path(&ContainerPath::Folder("db/campaign_variables_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    // Give the daracores extreme low priority so they don't overwrite other mods tables.
    campaign_variables.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));

    campaign_variables.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/campaign_variables_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    // Just in case another step of the launch process adds this table.
    campaign_variables.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/campaign_variables_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    // Sort them so file processing is done in the correct order.
    campaign_variables.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());

    let enc_extra_data = Some(EncodeableExtraData::new_from_game_info(game));
    let mut dec_extra_data = DecodeableExtraData::default();
    dec_extra_data.set_schema(Some(schema));
    let dec_extra_data = Some(dec_extra_data);

    for table in &mut campaign_variables {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            for row in data.data_mut() {

                if let Some(DecodedData::StringU8(key)) = row.first().cloned() {
                    if key == "max_traits" {
                        if let Some(DecodedData::F32(value)) = row.get_mut(1) {
                            *value = 999 as f32;
                        }
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    Ok(())
}

pub unsafe fn prepare_unit_multiplier(app_ui: &AppUI, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema, mod_paths: &[PathBuf]) -> Result<()> {
    let unit_multiplier = app_ui.actions_ui().unit_multiplier_spinbox().value();

    let mut kv_rules = vanilla_pack.files_by_path(&ContainerPath::Folder("db/_kv_rules_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let mut kv_unit_ability_scaling_rules = vanilla_pack.files_by_path(&ContainerPath::Folder("db/_kv_unit_ability_scaling_rules_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let mut land_units = vanilla_pack.files_by_path(&ContainerPath::Folder("db/land_units_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let mut main_units = vanilla_pack.files_by_path(&ContainerPath::Folder("db/main_units_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let mut unit_size_global_scalings = vanilla_pack.files_by_path(&ContainerPath::Folder("db/unit_size_global_scalings_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let mut unit_stat_to_size_scaling_values = vanilla_pack.files_by_path(&ContainerPath::Folder("db/unit_stat_to_size_scaling_values_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    // Give the daracores extreme low priority so they don't overwrite other mods tables.
    kv_rules.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));
    kv_unit_ability_scaling_rules.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));
    land_units.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));
    main_units.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));
    unit_size_global_scalings.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));
    unit_stat_to_size_scaling_values.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));

    kv_rules.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/_kv_rules_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    kv_unit_ability_scaling_rules.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/_kv_unit_ability_scaling_rules_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    land_units.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/land_units_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    main_units.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/main_units_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    unit_size_global_scalings.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/unit_size_global_scalings_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    unit_stat_to_size_scaling_values.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/unit_stat_to_size_scaling_values_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());


    kv_rules.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/_kv_rules_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    kv_unit_ability_scaling_rules.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/_kv_unit_ability_scaling_rules_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    land_units.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/land_units_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    main_units.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/main_units_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    unit_size_global_scalings.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/unit_size_global_scalings_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    unit_stat_to_size_scaling_values.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/unit_stat_to_size_scaling_values_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    // Sort them so file processing is done in the correct order.
    kv_rules.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    kv_unit_ability_scaling_rules.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    land_units.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    main_units.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    unit_size_global_scalings.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    unit_stat_to_size_scaling_values.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());

    // Decode each table, modify it, then re-encode it and add it.
    let enc_extra_data = Some(EncodeableExtraData::new_from_game_info(game));
    let mut dec_extra_data = DecodeableExtraData::default();
    dec_extra_data.set_schema(Some(schema));
    let dec_extra_data = Some(dec_extra_data);

    for table in &mut kv_rules {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            for row in data.data_mut() {

                if let Some(DecodedData::StringU8(key)) = row.first().cloned() {

                    // Battle width change.
                    if key == "unit_max_drag_width" {
                        if let Some(DecodedData::F32(value)) = row.get_mut(1) {
                            *value *= unit_multiplier as f32;
                        }
                    }

                    // Tomb kings campaign mechanic.
                    if key == "realm_of_souls_tier_1_death_threshold" || key == "realm_of_souls_tier_2_death_threshold" || key == "realm_of_souls_tier_3_death_threshold" {
                        if let Some(DecodedData::F32(value)) = row.get_mut(1) {
                            *value *= unit_multiplier as f32;
                        }
                    }

                    // Not sure what this do, but it seems to affect a few abilities.
                    if key == "unit_tier1_kills" || key == "unit_tier2_kills" || key == "unit_tier3_kills" {
                        if let Some(DecodedData::F32(value)) = row.get_mut(1) {
                            *value *= unit_multiplier as f32;
                        }
                    }

                    // Waaagh minimum threshold? Need to test this.
                    if key == "waaagh_base_threshold" {
                        if let Some(DecodedData::F32(value)) = row.get_mut(1) {
                            *value *= unit_multiplier as f32;
                        }
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    // Damage scaling.
    for table in &mut kv_unit_ability_scaling_rules {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            for row in data.data_mut() {
                if let Some(DecodedData::StringU8(key)) = row.first().cloned() {
                    if key == "direct_damage_large" || key == "direct_damage_medium" || key == "direct_damage_small" || key == "direct_damage_ultra" {
                        if let Some(DecodedData::F32(value)) = row.get_mut(1) {
                            *value *= unit_multiplier as f32;
                        }
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    // Some units like chariots may have multiple units on one engine, or on one mount. Here we do a pass to get the engine numbers,
    // to later calculate the men->engine and men->mount ratios.
    //
    // Otherwise, we may get weird stuff like 6 dark elven chariots with one chariot empty.
    let mut engine_amount = HashMap::new();
    let mut mount_amount = HashMap::new();
    for table in &mut land_units {
        if let Some(RFileDecoded::DB(data)) = table.decode(&dec_extra_data, false, true)? {
            let key_column = data.definition().column_position_by_name("key");
            let num_mounts_column = data.definition().column_position_by_name("num_mounts");
            let num_engines_column = data.definition().column_position_by_name("num_engines");
            for row in data.data().iter() {
                if let Some(key_column) = key_column {
                    if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {
                        let mut is_engine = false;

                        // Artillery pieces, chariots and weird units.
                        if let Some(column) = num_engines_column {
                            if let Some(DecodedData::I32(value)) = row.get(column) {
                                if *value != 0 {
                                    is_engine = true;
                                    if !engine_amount.contains_key(&key_value) {
                                        engine_amount.insert(key_value.to_owned(), *value);
                                    }
                                }
                            }
                        }

                        // Cavalry and some weird mounts, like sky junks.
                        if let Some(column) = num_mounts_column {
                            if let Some(DecodedData::I32(value)) = row.get(column) {
                                if !is_engine && *value > 0 && !mount_amount.contains_key(&key_value) {
                                    mount_amount.insert(key_value.to_owned(), *value);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Note: we need to process this before land_units to get the single entity units.
    let mut single_entity_units = HashSet::new();
    let mut processed_units = HashSet::new();
    for table in &mut main_units {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            let caste_column = data.definition().column_position_by_name("caste");
            let num_men_column = data.definition().column_position_by_name("num_men");
            let land_unit_column = data.definition().column_position_by_name("land_unit");
            let use_hitpoints_in_campaign_column = data.definition().column_position_by_name("use_hitpoints_in_campaign");

            for row in data.data_mut() {

                // General unit size.
                if let Some(num_men_column) = num_men_column {
                    if let Some(caste_column) = caste_column {
                        if let Some(use_hitpoints_in_campaign_column) = use_hitpoints_in_campaign_column {

                            // Store single entity units to increase their health later.
                            if let Some(land_unit_column) = land_unit_column {
                                if let Some(DecodedData::StringU8(land_unit_value)) = row.get(land_unit_column).cloned() {
                                    if let Some(DecodedData::StringU8(caste_value)) = row.get(caste_column).cloned() {
                                        if let Some(DecodedData::Boolean(hitpoins_in_campaign_value)) = row.get(use_hitpoints_in_campaign_column).cloned() {
                                            if let Some(DecodedData::I32(num_men_value)) = row.get_mut(num_men_column) {

                                                // There are some exceptions for this that need to be manually marked as single entities. Mainly:
                                                // - Lords & heroes.
                                                // - Anything marked as using hitpoints in campaign.
                                                if (caste_value == "lord" || caste_value == "hero" || hitpoins_in_campaign_value) && !processed_units.contains(&land_unit_value) {
                                                    single_entity_units.insert(land_unit_value.to_owned());
                                                }

                                                // If we have engines, we need to calculate the engine-men ratio to avoid ghost engines.
                                                else if let Some(engine_amount) = engine_amount.get(&land_unit_value) {
                                                    let new_engine_amount = (*engine_amount as f64 * unit_multiplier).round() as i32;
                                                    *num_men_value = (*num_men_value * new_engine_amount) / *engine_amount;
                                                    processed_units.insert(land_unit_value.to_owned());
                                                }

                                                // Same with some weird mounts.
                                                else if let Some(mount_amount) = mount_amount.get(&land_unit_value) {
                                                    let new_mount_amount = (*mount_amount as f64 * unit_multiplier).round() as i32;
                                                    *num_men_value = (*num_men_value * new_mount_amount) / *mount_amount;
                                                    processed_units.insert(land_unit_value.to_owned());
                                                }

                                                // If it's not a single entity, apply the multiplier.
                                                else {
                                                    *num_men_value = (*num_men_value as f64 * unit_multiplier).round() as i32;
                                                    processed_units.insert(land_unit_value.to_owned());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    for table in &mut land_units {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            let key_column = data.definition().column_position_by_name("key");
            let num_mounts_column = data.definition().column_position_by_name("num_mounts");
            let rank_depth_column = data.definition().column_position_by_name("rank_depth");
            let bonus_hit_points_column = data.definition().column_position_by_name("bonus_hit_points");
            let num_engines_column = data.definition().column_position_by_name("num_engines");

            for row in data.data_mut() {

                // For single entities, multiply their health, not their number too.
                // For engines with mounts (chariots) the calculatuion is different. We only need to increase engines, as mounts is mounts per-engine.
                if let Some(key_column) = key_column {
                    if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {
                        let is_single_entity = single_entity_units.get(&key_value).is_some();
                        let mut is_engine = false;

                        // Artillery pieces, chariots and weird units.
                        if let Some(column) = num_engines_column {
                            if let Some(DecodedData::I32(value)) = row.get_mut(column) {
                                if !is_single_entity {

                                    if *value != 0 {
                                        is_engine = true;
                                    }

                                    *value = (*value as f64 * unit_multiplier).round() as i32;
                                }
                            }
                        }

                        // Cavalry unit size (mounts).
                        if let Some(column) = num_mounts_column {
                            if let Some(DecodedData::I32(value)) = row.get_mut(column) {
                                if !is_single_entity && !is_engine {
                                    *value = (*value as f64 * unit_multiplier).round() as i32;
                                }
                            }
                        }

                        // Need to find out what the fuck is this.
                        if let Some(column) = rank_depth_column {
                            if let Some(DecodedData::I32(value)) = row.get_mut(column) {
                                if !is_single_entity {
                                    *value = (*value as f64 * unit_multiplier).round() as i32;
                                }
                            }
                        }

                        if is_single_entity {
                            if let Some(bonus_hit_points) = bonus_hit_points_column {
                                if let Some(DecodedData::I32(value)) = row.get_mut(bonus_hit_points) {
                                    *value = (*value as f64 * unit_multiplier).round() as i32;
                                }
                            }
                        }
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    // Generic stat scaling by battle and size.
    for table in &mut unit_size_global_scalings {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            let hit_points_building_small = data.definition().column_position_by_name("hit_points_building_small");
            let hit_points_building_medium = data.definition().column_position_by_name("hit_points_building_medium");
            let hit_points_building_large = data.definition().column_position_by_name("hit_points_building_large");
            let hit_points_building_ultra = data.definition().column_position_by_name("hit_points_building_ultra");
            let hit_points_siege_vehicle_small = data.definition().column_position_by_name("hit_points_siege_vehicle_small");
            let hit_points_siege_vehicle_medium = data.definition().column_position_by_name("hit_points_siege_vehicle_medium");
            let hit_points_siege_vehicle_large = data.definition().column_position_by_name("hit_points_siege_vehicle_large");
            let hit_points_siege_vehicle_ultra = data.definition().column_position_by_name("hit_points_siege_vehicle_ultra");
            let building_projectile_damage_small = data.definition().column_position_by_name("building_projectile_damage_small");
            let building_projectile_damage_medium = data.definition().column_position_by_name("building_projectile_damage_medium");
            let building_projectile_damage_large = data.definition().column_position_by_name("building_projectile_damage_large");
            let building_projectile_damage_ultra = data.definition().column_position_by_name("building_projectile_damage_ultra");
            let building_projectile_detonation_damage_small = data.definition().column_position_by_name("building_projectile_detonation_damage_small");
            let building_projectile_detonation_damage_medium = data.definition().column_position_by_name("building_projectile_detonation_damage_medium");
            let building_projectile_detonation_damage_large = data.definition().column_position_by_name("building_projectile_detonation_damage_large");
            let building_projectile_detonation_damage_ultra = data.definition().column_position_by_name("building_projectile_detonation_damage_ultra");
            let fort_tower_fire_frequency_small = data.definition().column_position_by_name("fort_tower_fire_frequency_small");
            let fort_tower_fire_frequency_medium = data.definition().column_position_by_name("fort_tower_fire_frequency_medium");
            let fort_tower_fire_frequency_large = data.definition().column_position_by_name("fort_tower_fire_frequency_large");
            let fort_tower_fire_frequency_ultra = data.definition().column_position_by_name("fort_tower_fire_frequency_ultra");

            for row in data.data_mut() {

                if let Some(column) = hit_points_building_small {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = hit_points_building_medium {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = hit_points_building_large {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = hit_points_building_ultra {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = hit_points_siege_vehicle_small {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = hit_points_siege_vehicle_medium {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = hit_points_siege_vehicle_large {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = hit_points_siege_vehicle_ultra {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = building_projectile_damage_small {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = building_projectile_damage_medium {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = building_projectile_damage_large {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = building_projectile_damage_ultra {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = building_projectile_detonation_damage_small {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = building_projectile_detonation_damage_medium {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = building_projectile_detonation_damage_large {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = building_projectile_detonation_damage_ultra {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = fort_tower_fire_frequency_small {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = fort_tower_fire_frequency_medium {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = fort_tower_fire_frequency_large {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }

                if let Some(column) = fort_tower_fire_frequency_ultra {
                    if let Some(DecodedData::F32(value)) = row.get_mut(column) {
                        *value *= unit_multiplier as f32;
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    // Generic stat scaling by size.
    for table in &mut unit_stat_to_size_scaling_values {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            let single_entity_value = data.definition().column_position_by_name("single_entity_value");
            for row in data.data_mut() {
                if let Some(single_entity_value_column) = single_entity_value {
                    if let Some(DecodedData::F64(value)) = row.get_mut(single_entity_value_column) {
                        *value *= unit_multiplier;
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    Ok(())
}

pub unsafe fn prepare_script_logging(reserved_pack: &mut Pack) -> Result<()> {
    let file = RFile::new_from_vec("why not working?!!".as_bytes(), FileType::Text, 0, SCRIPT_DEBUG_ACTIVATOR_PATH);
    reserved_pack.files_mut().insert(SCRIPT_DEBUG_ACTIVATOR_PATH.to_string(), file);

    Ok(())
}

pub unsafe fn prepare_skip_intro_videos(reserved_pack: &mut Pack) -> Result<()> {
    for path in INTRO_MOVIE_PATHS_BY_GAME {
        let file = RFile::new_from_vec(&EMPTY_CA_VP8, FileType::Video, 0, path);
        reserved_pack.files_mut().insert(path.to_string(), file);
    }

    Ok(())
}

pub unsafe fn prepare_universal_rebalancer(app_ui: &AppUI, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema, mod_paths: &[PathBuf]) -> Result<()> {
    let base_mod_id = app_ui.actions_ui().universal_rebalancer_combobox().current_text().to_std_string();
    let base_pack_path = (0..app_ui.pack_list_ui().model().row_count_0a())
        .find_map(|index| {
            let path = app_ui.pack_list_ui().model().item_2a(index, 2).text().to_std_string();
            if path.ends_with(&base_mod_id) {
                Some(path)
            } else {
                None
            }
        });

    match base_pack_path {
        Some(base_pack_path) => {

            let enc_extra_data = Some(EncodeableExtraData::new_from_game_info(game));
            let mut dec_extra_data = DecodeableExtraData::default();
            dec_extra_data.set_schema(Some(schema));
            let dec_extra_data = Some(dec_extra_data);

            let base_pack = Pack::read_and_merge(&[PathBuf::from(base_pack_path)], true, false)?;
            let mut land_units_base = base_pack.files_by_path(&ContainerPath::Folder("db/land_units_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .filter_map(|mut table| if let Ok(Some(RFileDecoded::DB(data))) = table.decode(&dec_extra_data, false, true) {
                    Some(data)
                } else {
                    None
                })
                .collect::<Vec<_>>();

            // Unlike with others options, we need first to get the files from the vanilla game, and from a single pack for doing calculations.
            let land_units_vanilla = vanilla_pack.files_by_path(&ContainerPath::Folder("db/land_units_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .filter_map(|mut table| if let Ok(Some(RFileDecoded::DB(data))) = table.decode(&dec_extra_data, false, true) {
                    if let Some(key_column) = data.definition().column_position_by_name("key") {
                        let hashed = data.data().par_iter()
                            .map(|row| (row[key_column].data_to_string().to_string(), row.to_vec()))
                            .collect::<HashMap<_,_>>();

                        Some((data, hashed))
                    } else {
                        None
                    }
                } else {
                    None
                })
                .collect::<Vec<_>>();

            // Generate the list of mod vs vanilla.
            let mut comparisons = HashMap::new();

            let mut land_unit_base_unit_keys = HashSet::new();
            for data in &mut land_units_base {
                let key_column = data.definition().column_position_by_name("key");
                let category_column = data.definition().column_position_by_name("category");
                let campaign_action_points_column = data.definition().column_position_by_name("campaign_action_points");
                let morale_column = data.definition().column_position_by_name("morale");
                let melee_attack_column = data.definition().column_position_by_name("melee_attack");
                let melee_defence_column = data.definition().column_position_by_name("melee_defence");
                let charge_bonus_column = data.definition().column_position_by_name("charge_bonus");
                let bonus_hit_points_column = data.definition().column_position_by_name("bonus_hit_points");
                let primary_ammo_column = data.definition().column_position_by_name("primary_ammo");
                let secondary_ammo_column = data.definition().column_position_by_name("secondary_ammo");
                let accuracy_column = data.definition().column_position_by_name("accuracy");
                let reload_column = data.definition().column_position_by_name("reload");
                let damage_mod_flame_column = data.definition().column_position_by_name("damage_mod_flame");
                let damage_mod_magic_column = data.definition().column_position_by_name("damage_mod_magic");
                let damage_mod_physical_column = data.definition().column_position_by_name("damage_mod_physical");
                let damage_mod_missile_column = data.definition().column_position_by_name("damage_mod_missile");
                let damage_mod_all_column = data.definition().column_position_by_name("damage_mod_all");
                let healing_power_column = data.definition().column_position_by_name("healing_power");
                let spell_mastery_column = data.definition().column_position_by_name("spell_mastery");
                let visibility_spotting_range_min_column = data.definition().column_position_by_name("visibility_spotting_range_min");
                let visibility_spotting_range_max_column = data.definition().column_position_by_name("visibility_spotting_range_max");
                let spot_dist_tree_column = data.definition().column_position_by_name("spot_dist_tree");
                let spot_dist_scrub_column = data.definition().column_position_by_name("spot_dist_scrub");
                let hiding_scalar_column = data.definition().column_position_by_name("hiding_scalar");

                for row in data.data().iter() {
                    if let Some(key_column) = key_column {
                        if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {
                            land_unit_base_unit_keys.insert(key_value.to_owned());

                            // Only use the first entry in case of duplicates.
                            if !comparisons.contains_key(&key_value) {

                                let mut cmp = UniversalRebalancerLandUnit::default();
                                cmp.key = key_value;

                                if let Some(column) = category_column {
                                    if let Some(DecodedData::StringU8(value)) = row.get(column) {
                                        cmp.category = value.to_owned();
                                    }
                                }

                                // Stats need to be find in both, base and vanilla.
                                if let Some(column) = campaign_action_points_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "campaign_action_points") {
                                            cmp.campaign_action_points = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = morale_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "morale") {
                                            cmp.morale = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = melee_attack_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "melee_attack") {
                                            cmp.melee_attack = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = melee_defence_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "melee_defence") {
                                            cmp.melee_defence = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = charge_bonus_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "charge_bonus") {
                                            cmp.charge_bonus = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = bonus_hit_points_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "bonus_hit_points") {
                                            cmp.bonus_hit_points = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = primary_ammo_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "primary_ammo") {
                                            cmp.primary_ammo = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = secondary_ammo_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "secondary_ammo") {
                                            cmp.secondary_ammo = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = accuracy_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "accuracy") {
                                            cmp.accuracy = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = reload_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "reload") {
                                            cmp.reload = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = damage_mod_flame_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "damage_mod_flame") {
                                            cmp.damage_mod_flame = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = damage_mod_magic_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "damage_mod_magic") {
                                            cmp.damage_mod_magic = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = damage_mod_physical_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "damage_mod_physical") {
                                            cmp.damage_mod_physical = (stat, *base_value);
                                        }
                                    }
                                }


                                if let Some(column) = damage_mod_missile_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "damage_mod_missile") {
                                            cmp.damage_mod_missile = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = damage_mod_all_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "damage_mod_all") {
                                            cmp.damage_mod_all = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = healing_power_column {
                                    if let Some(DecodedData::F32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_f32(&land_units_vanilla, cmp.key(), "healing_power") {
                                            cmp.healing_power = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = spell_mastery_column {
                                    if let Some(DecodedData::F32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_f32(&land_units_vanilla, cmp.key(), "spell_mastery") {
                                            cmp.spell_mastery = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = visibility_spotting_range_min_column {
                                    if let Some(DecodedData::F32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_f32(&land_units_vanilla, cmp.key(), "visibility_spotting_range_min") {
                                            cmp.visibility_spotting_range_min = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = visibility_spotting_range_max_column {
                                    if let Some(DecodedData::F32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_f32(&land_units_vanilla, cmp.key(), "visibility_spotting_range_max") {
                                            cmp.visibility_spotting_range_max = (stat, *base_value);
                                        }
                                    }
                                }


                                if let Some(column) = spot_dist_tree_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "spot_dist_tree") {
                                            cmp.spot_dist_tree = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = spot_dist_scrub_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_i32(&land_units_vanilla, cmp.key(), "spot_dist_scrub") {
                                            cmp.spot_dist_scrub = (stat, *base_value);
                                        }
                                    }
                                }

                                if let Some(column) = hiding_scalar_column {
                                    if let Some(DecodedData::F32(base_value)) = row.get(column) {
                                        if let Some(stat) = find_stat_in_table_f32(&land_units_vanilla, cmp.key(), "hiding_scalar") {
                                            cmp.hiding_scalar = (stat, *base_value);
                                        }
                                    }
                                }

                                comparisons.insert(cmp.key().to_owned(), cmp);
                            }
                        }
                    }
                }
            }

            // Once we get the comparison data, we need to match units to cultures and categories in order to split them into comparable groups.
            let mut main_units = vanilla_pack.files_by_path(&ContainerPath::Folder("db/main_units_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let mut units_custom_battle_permissions = vanilla_pack.files_by_path(&ContainerPath::Folder("db/units_custom_battle_permissions_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let mut factions = vanilla_pack.files_by_path(&ContainerPath::Folder("db/factions_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let mut cultures_subcultures = vanilla_pack.files_by_path(&ContainerPath::Folder("db/cultures_subcultures_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();

            // Give the daracores extreme low priority so they don't overwrite other mods tables.
            main_units.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));
            units_custom_battle_permissions.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));
            factions.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));
            cultures_subcultures.iter_mut().for_each(|x| rename_file_name_to_low_priority(x));

            main_units.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/main_units_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>());

            units_custom_battle_permissions.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/units_custom_battle_permissions_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>());

            factions.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/factions_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>());

            cultures_subcultures.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/cultures_subcultures_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>());

            main_units.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/main_units_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>());

            units_custom_battle_permissions.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/units_custom_battle_permissions_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>());

            factions.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/factions_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>());

            cultures_subcultures.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/cultures_subcultures_tables/".to_string()), true)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>());

            // Sort them so file processing is done in the correct order.
            main_units.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
            units_custom_battle_permissions.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
            factions.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
            cultures_subcultures.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());

            // Now, figure out what the culture is for each faction, as we can do that in one go.
            let mut sub_cul_to_cul = HashMap::new();
            for table in &mut cultures_subcultures {
                if let Some(RFileDecoded::DB(data)) = table.decode(&dec_extra_data, false, true)? {
                    let key_column = data.definition().column_position_by_name("subculture");
                    let cul_column = data.definition().column_position_by_name("culture");

                    for row in data.data().iter() {
                        if let Some(key_column) = key_column {
                            if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {
                                if !sub_cul_to_cul.contains_key(&key_value) {
                                    if let Some(column) = cul_column {
                                        if let Some(DecodedData::StringU8(cul)) = row.get(column) {
                                            sub_cul_to_cul.insert(key_value.to_owned(), cul.to_owned());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut fact_to_cul = HashMap::new();
            for table in &mut factions {
                if let Some(RFileDecoded::DB(data)) = table.decode(&dec_extra_data, false, true)? {
                    let key_column = data.definition().column_position_by_name("key");
                    let sc_column = data.definition().column_position_by_name("subculture");

                    for row in data.data().iter() {
                        if let Some(key_column) = key_column {
                            if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {
                                if !fact_to_cul.contains_key(&key_value) {
                                    if let Some(column) = sc_column {
                                        if let Some(DecodedData::StringU8(sub)) = row.get(column) {
                                            if let Some(cul) = sub_cul_to_cul.get(sub) {
                                                fact_to_cul.insert(key_value.to_owned(), cul.to_owned());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // While units can be in multiple faction/cultures... that's rare. We just pick the first culture.
            let mut main_unit_to_cul = HashMap::new();
            for table in &mut units_custom_battle_permissions {
                if let Some(RFileDecoded::DB(data)) = table.decode(&dec_extra_data, false, true)? {
                    let unit_column = data.definition().column_position_by_name("unit");
                    let faction_column = data.definition().column_position_by_name("faction");

                    for row in data.data().iter() {
                        if let Some(unit_column) = unit_column {
                            if let Some(DecodedData::StringU8(unit)) = row.get(unit_column).cloned() {
                                if !main_unit_to_cul.contains_key(&unit) {
                                    if let Some(faction_column) = faction_column {
                                        if let Some(DecodedData::StringU8(faction)) = row.get(faction_column) {
                                            if let Some(cul) = fact_to_cul.get(faction) {
                                                main_unit_to_cul.insert(unit.to_owned(), cul.to_owned());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut land_unit_to_cul = HashMap::new();
            for table in &mut main_units {
                if let Some(RFileDecoded::DB(data)) = table.decode(&dec_extra_data, false, true)? {
                    let main_unit_column = data.definition().column_position_by_name("unit");
                    let land_unit_column = data.definition().column_position_by_name("land_unit");

                    for row in data.data().iter() {
                        if let Some(land_unit_column) = land_unit_column {
                            if let Some(DecodedData::StringU8(land_unit)) = row.get(land_unit_column).cloned() {
                                if !land_unit_to_cul.contains_key(&land_unit) {
                                    if let Some(main_unit_column) = main_unit_column {
                                        if let Some(DecodedData::StringU8(main_unit)) = row.get(main_unit_column) {
                                            if let Some(cul) = main_unit_to_cul.get(main_unit) {
                                                land_unit_to_cul.insert(land_unit.to_owned(), cul.to_owned());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Now we split the units in culture/category groups for balancer calculations.
            let mut cmp_tree: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
            for (_, cmp) in &comparisons {

                // Ignore units that have no vanilla counterpart for balancing calculations.
                if let Some(cul) = land_unit_to_cul.get(cmp.key()) {

                    match cmp_tree.get_mut(cul) {
                        Some(cats) => {
                            match cats.get_mut(cmp.category()) {
                                Some(cat) => {
                                    if !cat.contains(cmp.key()) {
                                        cat.push(cmp.key().to_owned());
                                    }
                                }
                                None => {
                                    let mut cat = vec![];
                                    cat.push(cmp.key().to_owned());
                                    cats.insert(cmp.category().to_owned(), cat);
                                }
                            }
                        }
                        None => {
                            let mut cats = HashMap::new();
                            let mut cat = vec![];
                            cat.push(cmp.key().to_owned());
                            cats.insert(cmp.category().to_owned(), cat);
                            cmp_tree.insert(cul.to_owned(), cats);
                        }
                    }
                }
            }

            // Perform the calculations for each group.
            let mut averaged_categories_stats = HashMap::new();
            for (cul, categories) in cmp_tree {
                for (cat, units) in categories {
                    average_stat(&cul, &cat, "campaign_action_points", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "morale", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "melee_attack", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "melee_defence", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "charge_bonus", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "bonus_hit_points", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "primary_ammo", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "secondary_ammo", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "accuracy", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "reload", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "damage_mod_flame", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "damage_mod_magic", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "damage_mod_physical", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "damage_mod_missile", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "damage_mod_all", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "healing_power", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "spell_mastery", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "visibility_spotting_range_min", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "visibility_spotting_range_max", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "spot_dist_tree", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "spot_dist_scrub", &units, &comparisons, &mut averaged_categories_stats);
                    average_stat(&cul, &cat, "hiding_scalar", &units, &comparisons, &mut averaged_categories_stats);
                }
            }

            let mut a = averaged_categories_stats.iter().map(|(a, b)| (a, b)).collect::<Vec<_>>();
            a.sort_by_key(|a| a.0);

            // And finally, go over all units outside of the base mod (and outside mods that treat it as parent), and apply the avg multipliers.
            if !mod_paths.is_empty() {
                let packs_deps = mod_paths.iter()
                    .map(|path| {
                        let pack = Pack::read_and_merge(&[path.to_path_buf()], true, false).unwrap_or_default();
                        (pack.disk_file_name(), pack.dependencies().to_vec())
                    })
                    .collect::<HashMap<_,_>>();

                let mut land_units = modded_pack.files_by_path(&ContainerPath::Folder("db/land_units_tables/".to_string()), true)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>();

                land_units.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/land_units_tables/".to_string()), true)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>());

                for table in &mut land_units {

                    // If the table is neither the base pack nor a submod...
                    let cont_name = table.container_name().clone().unwrap();
                    if cont_name != base_pack.disk_file_name() &&
                        (
                            packs_deps.get(&cont_name).is_none() ||
                            !packs_deps.get(&cont_name).unwrap().contains(&base_pack.disk_file_name())
                        ) {

                        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
                            let key_column = data.definition().column_position_by_name("key");
                            let category_column = data.definition().column_position_by_name("category");
                            let campaign_action_points_column = data.definition().column_position_by_name("campaign_action_points");
                            let morale_column = data.definition().column_position_by_name("morale");
                            let melee_attack_column = data.definition().column_position_by_name("melee_attack");
                            let melee_defence_column = data.definition().column_position_by_name("melee_defence");
                            let charge_bonus_column = data.definition().column_position_by_name("charge_bonus");
                            let bonus_hit_points_column = data.definition().column_position_by_name("bonus_hit_points");
                            let primary_ammo_column = data.definition().column_position_by_name("primary_ammo");
                            let secondary_ammo_column = data.definition().column_position_by_name("secondary_ammo");
                            let accuracy_column = data.definition().column_position_by_name("accuracy");
                            let reload_column = data.definition().column_position_by_name("reload");
                            let damage_mod_flame_column = data.definition().column_position_by_name("damage_mod_flame");
                            let damage_mod_magic_column = data.definition().column_position_by_name("damage_mod_magic");
                            let damage_mod_physical_column = data.definition().column_position_by_name("damage_mod_physical");
                            let damage_mod_missile_column = data.definition().column_position_by_name("damage_mod_missile");
                            let damage_mod_all_column = data.definition().column_position_by_name("damage_mod_all");
                            let healing_power_column = data.definition().column_position_by_name("healing_power");
                            let spell_mastery_column = data.definition().column_position_by_name("spell_mastery");
                            let visibility_spotting_range_min_column = data.definition().column_position_by_name("visibility_spotting_range_min");
                            let visibility_spotting_range_max_column = data.definition().column_position_by_name("visibility_spotting_range_max");
                            let spot_dist_tree_column = data.definition().column_position_by_name("spot_dist_tree");
                            let spot_dist_scrub_column = data.definition().column_position_by_name("spot_dist_scrub");
                            let hiding_scalar_column = data.definition().column_position_by_name("hiding_scalar");

                            for row in data.data_mut() {
                                if let Some(key_column) = key_column {
                                    if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {

                                        // Only patch units not in the base mod.
                                        if !land_unit_base_unit_keys.contains(&key_value) {
                                            if let Some(cul) = land_unit_to_cul.get(&key_value) {

                                                if let Some(column) = category_column {
                                                    if let Some(DecodedData::StringU8(cat)) = row.get(column) {
                                                        let cul_cat = cul.to_owned() + cat;

                                                        if let Some(column) = campaign_action_points_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "campaign_action_points")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = morale_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "morale")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = melee_attack_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "melee_attack")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = melee_defence_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "melee_defence")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = charge_bonus_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "charge_bonus")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = bonus_hit_points_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "bonus_hit_points")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = primary_ammo_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "primary_ammo")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = secondary_ammo_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "secondary_ammo")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = accuracy_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "accuracy")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = reload_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "reload")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = damage_mod_flame_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "damage_mod_flame")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = damage_mod_magic_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "damage_mod_magic")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = damage_mod_physical_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "damage_mod_physical")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = damage_mod_missile_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "damage_mod_missile")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = damage_mod_all_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "damage_mod_all")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = healing_power_column {
                                                            if let Some(DecodedData::F32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "healing_power")) {
                                                                    *value *= multiplier;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = spell_mastery_column {
                                                            if let Some(DecodedData::F32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "spell_mastery")) {
                                                                    *value *= multiplier;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = visibility_spotting_range_min_column {
                                                            if let Some(DecodedData::F32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "visibility_spotting_range_min")) {
                                                                    *value *= multiplier;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = visibility_spotting_range_max_column {
                                                            if let Some(DecodedData::F32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "visibility_spotting_range_max")) {
                                                                    *value *= multiplier;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = spot_dist_tree_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "spot_dist_tree")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = spot_dist_scrub_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "spot_dist_scrub")) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
                                                                }
                                                            }
                                                        }
                                                        if let Some(column) = hiding_scalar_column {
                                                            if let Some(DecodedData::F32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&(cul_cat.to_owned() + "hiding_scalar")) {
                                                                    *value *= multiplier;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            table.set_decoded(RFileDecoded::DB(data))?;
                            table.encode(&enc_extra_data, false, true, false)?;
                            reserved_pack.insert(table.clone())?;

                        }
                    }
                }
            }

            Ok(())
        }
        None => Ok(()),
    }
}

fn find_stat_in_table_i32(tables: &[(DB, HashMap<String, Vec<DecodedData>>)], key: &str, stat: &str) -> Option<i32> {
    for (data, hashed) in tables {
        let stat_column = data.definition().column_position_by_name(stat);

        if let Some(row) = hashed.get(key) {
            if let Some(column) = stat_column {
                if let Some(DecodedData::I32(value)) = row.get(column) {
                    return Some(*value);
                }
            }
        }
    }

    None
}

fn find_stat_in_table_f32(tables: &[(DB, HashMap<String, Vec<DecodedData>>)], key: &str, stat: &str) -> Option<f32> {
    for (data, hashed) in tables {
        let stat_column = data.definition().column_position_by_name(stat);

        if let Some(row) = hashed.get(key) {
            if let Some(column) = stat_column {
                if let Some(DecodedData::F32(value)) = row.get(column) {
                    return Some(*value);
                }
            }
        }
    }

    None
}

fn average_stat(culture: &str, category: &str, stat: &str, units: &[String], cmps: &HashMap<String, UniversalRebalancerLandUnit>, averages: &mut HashMap<String, f32>) {
    let mut unit_count = 0.0;

    let mut avg_vanilla = 0.0;
    let mut avg_base = 0.0;

    for unit in units {
        if let Some(cmp) = cmps.get(unit) {
            match stat {
                "campaign_action_points" => {
                    avg_vanilla += cmp.campaign_action_points().0 as f32;
                    avg_base += cmp.campaign_action_points().1 as f32;
                }
                "morale" => {
                    avg_vanilla += cmp.morale().0 as f32;
                    avg_base += cmp.morale().1 as f32;
                }
                "melee_attack" => {
                    avg_vanilla += cmp.melee_attack().0 as f32;
                    avg_base += cmp.melee_attack().1 as f32;
                }
                "melee_defence" => {
                    avg_vanilla += cmp.melee_defence().0 as f32;
                    avg_base += cmp.melee_defence().1 as f32;
                }
                "charge_bonus" => {
                    avg_vanilla += cmp.charge_bonus().0 as f32;
                    avg_base += cmp.charge_bonus().1 as f32;
                }
                "bonus_hit_points" => {
                    avg_vanilla += cmp.bonus_hit_points().0 as f32;
                    avg_base += cmp.bonus_hit_points().1 as f32;
                }
                "primary_ammo" => {
                    avg_vanilla += cmp.primary_ammo().0 as f32;
                    avg_base += cmp.primary_ammo().1 as f32;
                }
                "secondary_ammo" => {
                    avg_vanilla += cmp.secondary_ammo().0 as f32;
                    avg_base += cmp.secondary_ammo().1 as f32;
                }
                "accuracy" => {
                    avg_vanilla += cmp.accuracy().0 as f32;
                    avg_base += cmp.accuracy().1 as f32;
                }
                "reload" => {
                    avg_vanilla += cmp.reload().0 as f32;
                    avg_base += cmp.reload().1 as f32;
                }
                "damage_mod_flame" => {
                    avg_vanilla += cmp.damage_mod_flame().0 as f32;
                    avg_base += cmp.damage_mod_flame().1 as f32;
                }
                "damage_mod_magic" => {
                    avg_vanilla += cmp.damage_mod_magic().0 as f32;
                    avg_base += cmp.damage_mod_magic().1 as f32;
                }
                "damage_mod_physical" => {
                    avg_vanilla += cmp.damage_mod_physical().0 as f32;
                    avg_base += cmp.damage_mod_physical().1 as f32;
                }
                "damage_mod_missile" => {
                    avg_vanilla += cmp.damage_mod_missile().0 as f32;
                    avg_base += cmp.damage_mod_missile().1 as f32;
                }
                "damage_mod_all" => {
                    avg_vanilla += cmp.damage_mod_all().0 as f32;
                    avg_base += cmp.damage_mod_all().1 as f32;
                }
                "healing_power" => {
                    avg_vanilla += cmp.healing_power().0;
                    avg_base += cmp.healing_power().1;
                }
                "spell_mastery" => {
                    avg_vanilla += cmp.spell_mastery().0;
                    avg_base += cmp.spell_mastery().1;
                }
                "visibility_spotting_range_min" => {
                    avg_vanilla += cmp.visibility_spotting_range_min().0;
                    avg_base += cmp.visibility_spotting_range_min().1;
                }
                "visibility_spotting_range_max" => {
                    avg_vanilla += cmp.visibility_spotting_range_max().0;
                    avg_base += cmp.visibility_spotting_range_max().1;
                }
                "spot_dist_tree" => {
                    avg_vanilla += cmp.spot_dist_tree().0 as f32;
                    avg_base += cmp.spot_dist_tree().1 as f32;
                }
                "spot_dist_scrub" => {
                    avg_vanilla += cmp.spot_dist_scrub().0 as f32;
                    avg_base += cmp.spot_dist_scrub().1 as f32;
                }
                "hiding_scalar" => {
                    avg_vanilla += cmp.hiding_scalar().0;
                    avg_base += cmp.hiding_scalar().1;
                }
                _ => continue,
            }
            unit_count += 1.0;
        }
    }

    // If there's no units in the category, skip it.
    if unit_count as i32 == 0 {
        return;
    }

    // Calculate the averages only if we actually have something that's not 0.
    if avg_vanilla as i32 != 0 {
        avg_vanilla /= unit_count;
    }

    if avg_base as i32 != 0 {
        avg_base /= unit_count;
    }

    // If the avgs are 0, don't bother dividing. Just put a 1.
    let avg_based_one = if avg_base as i32 == 0 || avg_vanilla as i32 == 0 {
        1.0
    } else {
        avg_base / avg_vanilla
    };

    averages.insert(culture.to_owned() + &category + stat, avg_based_one);
}
