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
use rpfm_lib::files::{Container, ContainerPath, DecodeableExtraData, EncodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded, table::DecodedData};
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
pub struct UniversalRebalancerUnitComparison {
    key: String,
    category: String,
    melee_attack: (i32, i32),
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

    let pack_names = mod_paths.iter().map(|path| path.file_name().unwrap().to_string_lossy().to_string()).collect::<Vec<_>>();
    reserved_pack.set_dependencies(pack_names);

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
            let mut land_units_vanilla = vanilla_pack.files_by_path(&ContainerPath::Folder("db/land_units_tables/".to_string()), true)
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
                let melee_attack_column = data.definition().column_position_by_name("melee_attack");

                for row in data.data().iter() {
                    if let Some(key_column) = key_column {
                        if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {
                            land_unit_base_unit_keys.insert(key_value.to_owned());

                            // Only use the first entry in case of duplicates.
                            if !comparisons.contains_key(&key_value) {

                                let mut cmp = UniversalRebalancerUnitComparison::default();
                                cmp.key = key_value;

                                if let Some(column) = category_column {
                                    if let Some(DecodedData::StringU8(value)) = row.get(column) {
                                        cmp.category = value.to_owned();
                                    }
                                }

                                // Stats need to be find in both, base and vanilla.
                                if let Some(column) = melee_attack_column {
                                    if let Some(DecodedData::I32(base_value)) = row.get(column) {

                                        for (data, hashed) in &mut land_units_vanilla {
                                            let stat_column = data.definition().column_position_by_name("melee_attack");

                                            if let Some(row) = hashed.get(cmp.key()) {
                                                if let Some(column) = stat_column {
                                                    if let Some(DecodedData::I32(vanilla_value)) = row.get(column) {
                                                        cmp.melee_attack = (*vanilla_value, *base_value);
                                                        break;
                                                    }
                                                }
                                            }
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
                    let mut unit_count = 0.0;

                    let mut avg_vanilla_melee_attack = 0.0;
                    let mut avg_base_melee_attack = 0.0;

                    for unit in &units {
                        if let Some(cmp) = comparisons.get(unit) {
                            avg_vanilla_melee_attack += cmp.melee_attack().0 as f32;
                            avg_base_melee_attack += cmp.melee_attack().1 as f32;
                            unit_count += 1.0;
                        }
                    }

                    avg_vanilla_melee_attack = avg_vanilla_melee_attack / unit_count;
                    avg_base_melee_attack = avg_base_melee_attack / unit_count;

                    let avg_based_one_melee_attack = avg_base_melee_attack / avg_vanilla_melee_attack;

                    averaged_categories_stats.insert(cul.to_owned() + &cat, avg_based_one_melee_attack);
                }
            }

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
                            let melee_attack_column = data.definition().column_position_by_name("melee_attack");

                            for row in data.data_mut() {
                                if let Some(key_column) = key_column {
                                    if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {

                                        // Only patch units not in the base mod.
                                        if !land_unit_base_unit_keys.contains(&key_value) {
                                            if let Some(cul) = land_unit_to_cul.get(&key_value) {

                                                if let Some(column) = category_column {
                                                    if let Some(DecodedData::StringU8(cat)) = row.get(column) {
                                                        let cul_cat = cul.to_owned() + cat;

                                                        // Melee attack.
                                                        if let Some(column) = melee_attack_column {
                                                            if let Some(DecodedData::I32(ref mut value)) = row.get_mut(column) {
                                                                if let Some(multiplier) = averaged_categories_stats.get(&cul_cat) {
                                                                    *value = (*value as f32 * multiplier).round() as i32;
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
