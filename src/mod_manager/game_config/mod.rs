//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing the centralized code for mod and load order management.

use anyhow::Result;
use getset::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::collections::{BTreeMap, HashMap};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use rpfm_lib::files::pack::Pack;
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType, supported_games::KEY_SHOGUN_2};
use rpfm_lib::integrations::log::error;

use rpfm_ui_common::settings::setting_string;

use crate::app_ui::{RESERVED_PACK_NAME, RESERVED_PACK_NAME_ALTERNATIVE};
use crate::mod_manager::{integrations::populate_mods, mods::Mod};
use crate::settings_ui::*;

mod versions;

const GAME_CONFIG_FILE_NAME_START: &str = "game_config_";
const GAME_CONFIG_FILE_NAME_END: &str = ".json";
pub const DEFAULT_CATEGORY: &str = "Unassigned";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GameConfig {

    // Key of the game.
    game_key: String,

    // Mods found for the game. Pack name is the key. This list contains all mods ever seen,
    // so if you reinstall a mod, it's data is reused.
    mods: HashMap<String, Mod>,

    // List of categories, and the pack names in each category.
    //
    // They are in order. Meaning if you want to change their order, you need to change them here.
    // And make sure only valid packs (with paths) are added.
    categories: BTreeMap<String, Vec<String>>,

    // List of categories in order.
    categories_order: Vec<String>,

    // TODO: Move the load order here, so it's always available and up to date.
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl GameConfig {

    pub fn load(game: &GameInfo, new_if_missing: bool) -> Result<Self> {
        let path = game_config_path()?.join(format!("{GAME_CONFIG_FILE_NAME_START}{}{GAME_CONFIG_FILE_NAME_END}", game.key()));
        if !path.is_file() && new_if_missing {
            return Ok(Self {
                game_key: game.key().to_string(),
                ..Default::default()
            });
        }

        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;

        let profile: Self = serde_json::from_slice(&data)?;

        Ok(profile)
    }

    pub fn save(&mut self, game: &GameInfo) -> Result<()> {
        let path = game_config_path()?.join(format!("{GAME_CONFIG_FILE_NAME_START}{}{GAME_CONFIG_FILE_NAME_END}", game.key()));

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update(game_name: &str) -> Result<()> {
        let _ = versions::v0::GameConfigV0::update(game_name);
        let _ = versions::v1::GameConfigV1::update(game_name);
        let _ = versions::v2::GameConfigV2::update(game_name);
        let _ = versions::v3::GameConfigV3::update(game_name);

        Ok(())
    }

    // TODO: Optimize this if it gets too slow.
    pub fn category_for_mod(&self, id: &str) -> String {
        let mut category = DEFAULT_CATEGORY.to_string();
        let mut found = false;
        for (cat, packs) in &self.categories {
            for pack in packs {
                if pack == id {
                    category = cat.to_owned();
                    found = true;
                    break;
                }
            }
        }

        // If the mod wasn't found, it's a bug.
        if !found {
            error!("Mod {} not found in a category. This is a bug in the code that parses the mods, or you passed a mod which is not installed.", id);
        }

        category
    }

    pub fn create_category(&mut self, category: &str) {
        self.categories_mut().insert(category.to_owned(), vec![]);

        let pos = if !self.categories_order().is_empty() {
            self.categories_order().len() - 1
        } else {
            0
        };
        self.categories_order_mut().insert(pos, category.to_owned());
    }

    pub fn delete_category(&mut self, category: &str) {

        // Just in case we don't have a default category yet.
        if self.categories().get(DEFAULT_CATEGORY).is_none() {
            self.categories_mut().insert(DEFAULT_CATEGORY.to_owned(), vec![]);
        }

        // Do not delete default category.
        if category == DEFAULT_CATEGORY {
            return;
        }

        self.categories_mut().remove(category);
        self.categories_order_mut().retain(|x| x != category);
    }

    pub fn update_mod_list(&mut self, game: &GameInfo, game_path: &Path, skip_network_update: bool) -> Result<()> {

        // Get the modified date of the game's exe, to check if a mod is outdated or not.
        let last_update_date = if let Some(exe_path) = game.executable_path(game_path) {
            if let Ok(exe) = File::open(exe_path) {
                exe.metadata()?.created()?.duration_since(UNIX_EPOCH)?.as_secs()
            } else {
                0
            }
        } else {
            0
        };

        // Clear the mod paths, just in case a failure while loading them leaves them unclean.
        self.mods_mut().values_mut().for_each(|modd| modd.paths_mut().clear());

        // If we have a path, load all the mods to the UI.
        if game_path.components().count() > 1 && game_path.is_dir() {

            // Vanilla paths may fail if the game path is incorrect, or the game is not properly installed.
            // In that case, we assume there are no packs nor mods to load to avoid further errors.
            if let Ok(vanilla_packs) = game.ca_packs_paths(game_path) {
                let data_paths = game.data_packs_paths(game_path);
                let content_paths = game.content_packs_paths(game_path);

                let mut steam_ids = vec![];

                // Initialize the mods in the contents folders first.
                //
                // These have less priority.
                if let Some(ref paths) = content_paths {
                    let packs = paths.par_iter()
                        .map(|path| (path, Pack::read_and_merge(&[path.to_path_buf()], true, false)))
                        .collect::<Vec<_>>();

                    for (path, pack) in packs {
                        let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                        if let Ok(pack) = pack {
                            if pack.pfh_file_type() == PFHFileType::Mod || pack.pfh_file_type() == PFHFileType::Movie {
                                match self.mods_mut().get_mut(&pack_name) {
                                    Some(modd) => {
                                        if !modd.paths().contains(path) {
                                            modd.paths_mut().push(path.to_path_buf());
                                        }

                                        // Get the steam id from the path, if possible.
                                        let steam_id = path.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();
                                        steam_ids.push(steam_id.to_owned());
                                        modd.set_steam_id(Some(steam_id));
                                        modd.set_pack_type(pack.pfh_file_type());

                                        let metadata = modd.paths().last().unwrap().metadata()?;
                                        modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_outdated(last_update_date > *modd.time_updated() as u64);
                                    }
                                    None => {
                                        let mut modd = Mod::default();
                                        modd.set_name(pack_name.to_owned());
                                        modd.set_id(pack_name.to_owned());
                                        modd.set_paths(vec![path.to_path_buf()]);
                                        modd.set_pack_type(pack.pfh_file_type());

                                        let metadata = modd.paths()[0].metadata()?;
                                        modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_outdated(last_update_date > *modd.time_updated() as u64);

                                        // Get the steam id from the path, if possible.
                                        let steam_id = path.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();
                                        steam_ids.push(steam_id.to_owned());
                                        modd.set_steam_id(Some(steam_id));

                                        self.mods_mut().insert(pack_name, modd);
                                    }
                                }
                            }
                        }
                    }
                }

                // Ignore network population errors for now.
                if !skip_network_update {
                    let _ = populate_mods(self.mods_mut(), &steam_ids, last_update_date);
                }

                // If any of the mods has a .bin file, we need to copy it to /data and turn it into a Pack.
                // All the if lets are because we only want to do all this if nothing files and ignore failures.
                let steam_user_id = setting_string("steam_user_id");
                for modd in self.mods_mut().values_mut() {
                    if let Some(last_path) = modd.paths().last() {
                        if let Some(extension) = last_path.extension() {

                            // Only copy bins which are not yet in the data folder and which are not made by the steam user.
                            // If the game is Shogun 2, also copy packs to data. Shogun 2 doesn't support loading packs from outside /data.
                            let legacy_mod = extension.to_string_lossy() == "bin" && !modd.file_name().is_empty();
                            if legacy_mod || (extension.to_string_lossy() == "pack" && game.key() == KEY_SHOGUN_2) {
                                if let Ok(mut pack) = Pack::read_and_merge(&[last_path.to_path_buf()], true, false) {
                                    if let Ok(new_path) = game.data_path(game_path) {

                                        // Filename is only populated on legacy bin files.
                                        if legacy_mod {
                                            if let Some(name) = modd.file_name().split('/').last() {
                                                let new_path = new_path.join(name);

                                                // Copy the files unless it exists and its ours.
                                                if (!new_path.is_file() || (new_path.is_file() && &steam_user_id != modd.creator())) && pack.save(Some(&new_path), game, &None).is_ok() {
                                                    modd.paths_mut().insert(0, new_path);
                                                }
                                            }
                                        }

                                        // Alternative logic for normal packs.
                                        else {
                                            let new_path = new_path.join(modd.id());

                                            // Copy the files unless it exists and its ours.
                                            if (!new_path.is_file() || (new_path.is_file() && &steam_user_id != modd.creator())) && pack.save(Some(&new_path), game, &None).is_ok() {
                                                modd.paths_mut().insert(0, new_path);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(ref paths) = data_paths {
                    let paths = paths.iter()
                        .filter(|path| {
                            if let Ok(canon_path) = std::fs::canonicalize(path) {
                                !vanilla_packs.contains(&canon_path) &&
                                    canon_path.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(String::new) != RESERVED_PACK_NAME &&
                                    canon_path.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(String::new) != RESERVED_PACK_NAME_ALTERNATIVE
                            } else {
                                false
                            }
                        })
                        .collect::<Vec<_>>();

                    let packs = paths.par_iter()
                        .map(|path| (path, Pack::read_and_merge(&[path.to_path_buf()], true, false)))
                        .collect::<Vec<_>>();

                    for (path, pack) in packs {
                        let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                        if let Ok(pack) = pack {
                            if pack.pfh_file_type() == PFHFileType::Mod || pack.pfh_file_type() == PFHFileType::Movie {

                                // Check if the pack corresponds to a bin.
                                if let Some((_, modd)) = self.mods_mut().iter_mut().find(|(_, modd)| !modd.file_name().is_empty() && modd.file_name().split('/').last().unwrap() == pack_name) {
                                    if !modd.paths().contains(path) {
                                        modd.paths_mut().insert(0, path.to_path_buf());
                                    }

                                    let metadata = modd.paths()[0].metadata()?;
                                    modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                    modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                    modd.set_outdated(last_update_date > *modd.time_updated() as u64);
                                } else {
                                    match self.mods_mut().get_mut(&pack_name) {
                                        Some(modd) => {
                                            if !modd.paths().contains(path) {
                                                modd.paths_mut().insert(0, path.to_path_buf());
                                            }
                                            modd.set_pack_type(pack.pfh_file_type());

                                            let metadata = modd.paths()[0].metadata()?;
                                            modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_outdated(last_update_date > *modd.time_updated() as u64);
                                        }
                                        None => {
                                            let mut modd = Mod::default();
                                            modd.set_name(pack_name.to_owned());
                                            modd.set_id(pack_name.to_owned());
                                            modd.set_paths(vec![path.to_path_buf()]);
                                            modd.set_pack_type(pack.pfh_file_type());

                                            let metadata = modd.paths()[0].metadata()?;
                                            modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_outdated(last_update_date > *modd.time_updated() as u64);
                                            self.mods_mut().insert(pack_name, modd);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update the categories list to remove any mod that has no path, and add any new mod to the default category.
        for mods in self.categories.values_mut() {
            mods.retain(|mod_id| match self.mods.get(mod_id) {
                Some(modd) => !modd.paths().is_empty(),
                None => false,
            });
        }

        let mut mods_to_add = vec![];
        for modd in self.mods.values() {
            if !modd.paths().is_empty() && self.categories().iter().all(|(_, mods)| !mods.contains(modd.id())) {
                mods_to_add.push(modd.id().to_owned());
            }
        }

        match self.categories_mut().get_mut(DEFAULT_CATEGORY) {
            Some(mods) => mods.append(&mut mods_to_add),
            None => { self.categories_mut().insert(DEFAULT_CATEGORY.to_owned(), mods_to_add); },
        }

        // If we got a default category, make sure it's always at the end.
        if let Some(cat) = self.categories_order().last() {
            if cat != DEFAULT_CATEGORY && self.categories().get(DEFAULT_CATEGORY).is_some() {
                if let Some(mods) = self.categories_mut().remove(DEFAULT_CATEGORY) {
                    self.categories_mut().insert(DEFAULT_CATEGORY.to_owned(), mods);
                }
            }
        }

        // Save the GameConfig or we may lost the population.
        self.save(game)
    }
}
