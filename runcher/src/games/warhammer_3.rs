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

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rpfm_lib::schema::Schema;
use rpfm_lib::files::{Container, ContainerPath, DecodeableExtraData, EncodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::GameInfo;

use crate::app_ui::AppUI;
use crate::games::EMPTY_CA_VP8;

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
//                             Implementations
//-------------------------------------------------------------------------------//


pub unsafe fn prepare_trait_limit_removal(app_ui: &AppUI, game: &GameInfo, game_path: &Path, reserved_pack: &mut Pack, schema: &Schema) -> Result<()> {
    let vanilla_pack = Pack::read_and_merge_ca_packs(game, game_path)?;
    let mut campaign_variables = vanilla_pack.files_by_path(&ContainerPath::Folder("db/campaign_variables_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let paths = (0..app_ui.pack_list_ui().model().row_count_0a())
        .map(|index| PathBuf::from(app_ui.pack_list_ui().model().item_2a(index, 2).text().to_std_string()))
        .collect::<Vec<_>>();

    if !paths.is_empty() {
        let modded_pack = Pack::read_and_merge(&paths, true, false)?;
        campaign_variables.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/campaign_variables_tables/".to_string()), true)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>());
    }

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

pub unsafe fn prepare_unit_multiplier(app_ui: &AppUI, game: &GameInfo, game_path: &Path, reserved_pack: &mut Pack, schema: &Schema) -> Result<()> {
    let unit_multiplier = app_ui.actions_ui().unit_multiplier_spinbox().value();

    let vanilla_pack = Pack::read_and_merge_ca_packs(game, game_path)?;
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

    let paths = (0..app_ui.pack_list_ui().model().row_count_0a())
        .map(|index| PathBuf::from(app_ui.pack_list_ui().model().item_2a(index, 2).text().to_std_string()))
        .collect::<Vec<_>>();

    if !paths.is_empty() {
        let modded_pack = Pack::read_and_merge(&paths, true, false)?;
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
    }

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

    let pack_names = paths.iter().map(|path| path.file_name().unwrap().to_string_lossy().to_string()).collect::<Vec<_>>();
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
