//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rpfm_lib::schema::Schema;
use rpfm_lib::files::{Container, ContainerPath, DecodeableExtraData, EncodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::GameInfo;

use crate::app_ui::AppUI;
use crate::games::EMPTY_CA_VP8;

const INTRO_MOVIE_PATHS_BY_GAME: [&str; 2] = [
    "movies/startup_movie_01.ca_vp8",
    "movies/startup_movie_02.ca_vp8",
];

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub unsafe fn prepare_unit_multiplier(app_ui: &AppUI, game: &GameInfo, game_path: &Path, reserved_pack: &mut Pack, schema: &Schema) -> Result<()> {
    let unit_multiplier = app_ui.actions_ui().unit_multiplier_spinbox().value();

    let vanilla_pack = Pack::read_and_merge_ca_packs(game, game_path)?;
    let mut kv_key_buildings = vanilla_pack.files_by_path(&ContainerPath::Folder("db/_kv_key_buildings_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

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

    let mut land_units_templates = vanilla_pack.files_by_path(&ContainerPath::Folder("db/land_units_templates_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let paths = (0..app_ui.pack_list_ui().model().row_count_0a())
        .map(|index| PathBuf::from(app_ui.pack_list_ui().model().item_2a(index, 2).text().to_std_string()))
        .collect::<Vec<_>>();

    if !paths.is_empty() {
        let modded_pack = Pack::read_and_merge(&paths, true, false)?;
        kv_key_buildings.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/_kv_key_buildings_tables/".to_string()), true)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>());

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

        land_units_templates.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/land_units_templates_tables/".to_string()), true)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>());
    }

    // Sort them so file processing is done in the correct order.
    kv_key_buildings.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    kv_rules.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    kv_unit_ability_scaling_rules.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    land_units.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());
    land_units_templates.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());

    // Decode each table, modify it, then re-encode it and add it.
    let enc_extra_data = Some(EncodeableExtraData::new_from_game_info(game));
    let mut dec_extra_data = DecodeableExtraData::default();
    dec_extra_data.set_schema(Some(schema));
    let dec_extra_data = Some(dec_extra_data);

    for table in &mut kv_key_buildings {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            for row in data.data_mut() {

                if let Some(DecodedData::StringU8(key)) = row.first().cloned() {

                    // Fort tower frequency.
                    if key == "fort_tower_fire_frequency_small" || key == "fort_tower_fire_frequency_medium" || key == "fort_tower_fire_frequency_large" || key == "fort_tower_fire_frequency_ultra" || key == "fort_tower_fire_frequency_extreme" {
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

                    // Not sure what this do, but it seems to affect a few abilities.
                    if key == "unit_tier1_kills" || key == "unit_tier2_kills" || key == "unit_tier3_kills" {
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
                    if key == "direct_damage_large" || key == "direct_damage_medium" || key == "direct_damage_small" || key == "direct_damage_ultra" || key == "direct_damage_extreme" {
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

    // This is simpler than in other games: all units are defined as single units, and this table combines them into a single land unit and multiplies them to have the amount of soldiers needed.
    //
    // The only thing to take into account is that units that are only one soldier needs to be saved so we know not to increase it's rank depth.
    let mut single_entity_units = HashSet::new();
    for table in &mut land_units_templates {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            let land_unit_column = data.definition().column_position_by_name("land_unit");
            let num_composed_entities_column = data.definition().column_position_by_name("num_composed_entities");
            let hp_pool_column = data.definition().column_position_by_name("hp_pool");

            let mut key_count: HashMap<String, i32> = HashMap::new();
            for row in data.data_mut() {

                if let Some(land_unit_column) = land_unit_column {
                    if let Some(DecodedData::StringU8(land_unit)) = row.get(land_unit_column).cloned() {

                        // Number of men in a unit.
                        if let Some(column) = num_composed_entities_column {
                            if let Some(DecodedData::I32(value)) = row.get_mut(column) {

                                // Add them to the list.
                                match key_count.get_mut(&land_unit) {
                                    Some(data) => *data += *value,
                                    None => { key_count.insert(land_unit.to_string(), *value); }
                                }

                                // Ignore single units (heroes and unit captains).
                                if *value > 1 {
                                    *value = (*value as f64 * unit_multiplier).round() as i32;
                                }
                            }
                        }
                    }
                }

                // HP pool of the whole unit.
                if let Some(column) = hp_pool_column {
                    if let Some(DecodedData::I32(value)) = row.get_mut(column) {
                        *value = (*value as f64 * unit_multiplier).round() as i32;
                    }
                }
            }

            single_entity_units = key_count.par_iter()
                .filter_map(|(key, count)| {
                    if *count == 1 {
                        Some(key.to_owned())
                    } else {
                        None
                    }
                }).collect();

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    // Rank depth.
    for table in &mut land_units {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            let key_column = data.definition().column_position_by_name("key");
            let rank_depth_column = data.definition().column_position_by_name("rank_depth");

            for row in data.data_mut() {
                if let Some(key_column) = key_column {
                    if let Some(DecodedData::StringU8(key_value)) = row.get(key_column).cloned() {

                        if single_entity_units.get(&key_value).is_none() {
                            if let Some(column) = rank_depth_column {
                                if let Some(DecodedData::I32(value)) = row.get_mut(column) {
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

    let pack_names = paths.iter().map(|path| path.file_name().unwrap().to_string_lossy().to_string()).collect::<Vec<_>>();
    reserved_pack.set_dependencies(pack_names);

    Ok(())
}

pub unsafe fn prepare_skip_intro_videos(reserved_pack: &mut Pack) -> Result<()> {
    for path in INTRO_MOVIE_PATHS_BY_GAME {
        let file = RFile::new_from_vec(&EMPTY_CA_VP8, FileType::Video, 0, path);
        reserved_pack.files_mut().insert(path.to_string(), file);
    }

    Ok(())
}
