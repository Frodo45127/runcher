//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing the centralized code for mod and load order management.

use anyhow::Result;
use crossbeam::channel::Receiver;
use getset::*;
use rayon::{iter::Either, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::collections::{BTreeMap, HashMap};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use rpfm_lib::files::pack::Pack;
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType};
use rpfm_lib::integrations::log::error;

use crate::games::{RESERVED_PACK_NAME, RESERVED_PACK_NAME_ALTERNATIVE};
use crate::communications::{Command, Response};
use crate::mod_manager::{load_order::LoadOrder, mods::Mod};
use crate::{settings_ui::*, CENTRAL_COMMAND};

use super::secondary_mods_packs_paths;

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
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl GameConfig {

    pub fn load(game: &GameInfo, new_if_missing: bool) -> Result<Self> {
        let path = game_config_path()?.join(format!("{GAME_CONFIG_FILE_NAME_START}{}{GAME_CONFIG_FILE_NAME_END}", game.key()));
        if !path.is_file() && new_if_missing {
            let mut config = Self {
                game_key: game.key().to_string(),
                ..Default::default()
            };

            config.categories_mut().insert(DEFAULT_CATEGORY.to_owned(), vec![]);
            config.categories_order_mut().push(DEFAULT_CATEGORY.to_owned());

            return Ok(config);
        }

        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;

        let mut config: Self = serde_json::from_slice(&data)?;

        // Just in case we don't have a default category yet.
        if config.categories().get(DEFAULT_CATEGORY).is_none() {
            config.categories_mut().insert(DEFAULT_CATEGORY.to_owned(), vec![]);
            config.categories_order_mut().retain(|category| category != DEFAULT_CATEGORY);
            config.categories_order_mut().push(DEFAULT_CATEGORY.to_owned());
        }

        Ok(config)
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
        //let _ = versions::v0::GameConfigV0::update(game_name);
        //let _ = versions::v1::GameConfigV1::update(game_name);
        //let _ = versions::v2::GameConfigV2::update(game_name);
        //let _ = versions::v3::GameConfigV3::update(game_name);
        let _ = versions::v4::GameConfigV4::update(game_name);

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

    /// NOTE: This returns a channel receiver for the workshop/equivalent service data request.
    /// This is done so the request doesn't hang the entire load process, as it usually takes 2 or 3 seconds to complete.
    pub fn update_mod_list(&mut self, game: &GameInfo, game_path: &Path, load_order: &mut LoadOrder, skip_network_update: bool) -> Result<Option<Receiver<Response>>> {
        let mut receiver = None;

        // Clear the mod paths, just in case a failure while loading them leaves them unclean.
        self.mods_mut().values_mut().for_each(|modd| modd.paths_mut().clear());

        // If we have a path, load all the mods to the UI.
        if game_path.components().count() > 1 && game_path.is_dir() {

            // Vanilla paths may fail if the game path is incorrect, or the game is not properly installed.
            // In that case, we assume there are no packs nor mods to load to avoid further errors.
            if let Ok(vanilla_packs) = game.ca_packs_paths(game_path) {
                let data_paths = game.data_packs_paths(game_path);
                let content_path = game.content_path(game_path).map(|path| std::fs::canonicalize(path.clone()).unwrap_or(path));
                let content_paths = game.content_packs_paths(game_path);
                let secondary_mods_paths = secondary_mods_packs_paths(game.key());

                let mut steam_ids = vec![];

                // Initialize the mods in the contents folders first.
                //
                // These have less priority.
                if let Ok(ref content_path) = content_path {
                    if let Some(ref paths) = content_paths {
                        let (packs, maps): (Vec<_>, Vec<_>) = paths.par_iter()
                            .partition_map(|path| match Pack::read_and_merge(&[path.to_path_buf()], true, false, false) {
                                Ok(pack) => Either::Left((path, pack)),
                                Err(_) => Either::Right(path),
                            });

                        for (path, pack) in packs {
                            let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                            if pack.pfh_file_type() == PFHFileType::Mod || pack.pfh_file_type() == PFHFileType::Movie {
                                match self.mods_mut().get_mut(&pack_name) {
                                    Some(modd) => {
                                        if !modd.paths().contains(path) {
                                            modd.paths_mut().push(path.to_path_buf());
                                        }

                                        modd.set_pack_type(pack.pfh_file_type());

                                        let metadata = modd.paths().last().unwrap().metadata()?;
                                        #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);

                                        // Get the steam id from the path, if possible.
                                        let path_strip = path.strip_prefix(content_path)?.to_string_lossy().replace("\\", "/");
                                        let path_strip_split = path_strip.split("/").collect::<Vec<_>>();
                                        if !path_strip_split.is_empty() {
                                            let steam_id = path_strip_split[0].to_owned();
                                            steam_ids.push(steam_id.to_owned());
                                            modd.set_steam_id(Some(steam_id));
                                        }
                                    }
                                    None => {
                                        let mut modd = Mod::default();
                                        modd.set_name(pack_name.to_owned());
                                        modd.set_id(pack_name.to_owned());
                                        modd.set_paths(vec![path.to_path_buf()]);
                                        modd.set_pack_type(pack.pfh_file_type());

                                        let metadata = modd.paths()[0].metadata()?;
                                        #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);

                                        // Get the steam id from the path, if possible.
                                        let path_strip = path.strip_prefix(content_path)?.to_string_lossy().replace("\\", "/");
                                        let path_strip_split = path_strip.split("/").collect::<Vec<_>>();
                                        if !path_strip_split.is_empty() {
                                            let steam_id = path_strip_split[0].to_owned();
                                            steam_ids.push(steam_id.to_owned());
                                            modd.set_steam_id(Some(steam_id));
                                        }

                                        self.mods_mut().insert(pack_name, modd);
                                    }
                                }
                            }
                        }

                        // Maps use their own logic.
                        for path in &maps {
                            let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                            if let Some(extension) = path.extension() {
                                if extension == "bin" {
                                    let mut file = BufReader::new(File::open(path)?);
                                    let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
                                    file.read_to_end(&mut data)?;

                                    let reader = BufReader::new(Cursor::new(data.to_vec()));
                                    let mut decompressor = flate2::read::ZlibDecoder::new(reader);
                                    let mut data_dec = vec![];

                                    // If they got decompressed correctly, we assume is a map. Shogun 2 64-bit update not only broke extracting the maps, but also
                                    // loading them from /maps. So instead we treat them like mods, and we generate their Pack once we get their Steam.
                                    if decompressor.read_to_end(&mut data_dec).is_ok() {
                                        match self.mods_mut().get_mut(&pack_name) {
                                            Some(modd) => {
                                                if !modd.paths().contains(path) {
                                                    modd.paths_mut().push(path.to_path_buf());
                                                }

                                                modd.set_pack_type(PFHFileType::Mod);

                                                let metadata = modd.paths().last().unwrap().metadata()?;
                                                #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);

                                                // Get the steam id from the path, if possible.
                                                let path_strip = path.strip_prefix(content_path)?.to_string_lossy().replace("\\", "/");
                                                let path_strip_split = path_strip.split("/").collect::<Vec<_>>();
                                                if !path_strip_split.is_empty() {
                                                    let steam_id = path_strip_split[0].to_owned();
                                                    steam_ids.push(steam_id.to_owned());
                                                    modd.set_steam_id(Some(steam_id));
                                                }
                                            }
                                            None => {
                                                let mut modd = Mod::default();
                                                modd.set_name(pack_name.to_owned());
                                                modd.set_id(pack_name.to_owned());
                                                modd.set_paths(vec![path.to_path_buf()]);
                                                modd.set_pack_type(PFHFileType::Mod);

                                                let metadata = modd.paths()[0].metadata()?;
                                                #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);

                                                // Get the steam id from the path, if possible.
                                                let path_strip = path.strip_prefix(content_path)?.to_string_lossy().replace("\\", "/");
                                                let path_strip_split = path_strip.split("/").collect::<Vec<_>>();
                                                if !path_strip_split.is_empty() {
                                                    let steam_id = path_strip_split[0].to_owned();
                                                    steam_ids.push(steam_id.to_owned());
                                                    modd.set_steam_id(Some(steam_id));
                                                }

                                                self.mods_mut().insert(pack_name, modd);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Ignore network population errors for now.
                if !skip_network_update {
                    receiver = Some(CENTRAL_COMMAND.send_network(Command::RequestModsData(Box::new(game.clone()), steam_ids)));
                }

                // Then, if the game supports secondary mod path (only since Shogun 2) we check for mods in there. These have middle priority.
                //
                // Non supported games will simply return "None" here.
                if let Some(ref paths) = secondary_mods_paths {
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
                        .map(|path| (path, Pack::read_and_merge(&[path.to_path_buf()], true, false, false)))
                        .collect::<Vec<_>>();

                    for (path, pack) in packs {
                        let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                        if let Ok(pack) = pack {
                            if pack.pfh_file_type() == PFHFileType::Mod || pack.pfh_file_type() == PFHFileType::Movie {

                                match self.mods_mut().get_mut(&pack_name) {
                                    Some(modd) => {
                                        if !modd.paths().contains(path) {
                                            modd.paths_mut().insert(0, path.to_path_buf());
                                        }
                                        modd.set_pack_type(pack.pfh_file_type());

                                        let metadata = modd.paths()[0].metadata()?;
                                        #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                    }
                                    None => {

                                        // If the mod fails to be found, is possible is a legacy mod. Find it by alt name.
                                        match self.mods_mut().values_mut()
                                            .filter(|modd| modd.alt_name().is_some())
                                            .find(|modd| modd.alt_name().unwrap() == pack_name) {

                                            Some(modd) => {
                                                if !modd.paths().contains(path) {
                                                    modd.paths_mut().insert(0, path.to_path_buf());
                                                }
                                                modd.set_pack_type(pack.pfh_file_type());

                                                let metadata = modd.paths()[0].metadata()?;
                                                #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            }

                                            None => {

                                                let mut modd = Mod::default();
                                                modd.set_name(pack_name.to_owned());
                                                modd.set_id(pack_name.to_owned());
                                                modd.set_paths(vec![path.to_path_buf()]);
                                                modd.set_pack_type(pack.pfh_file_type());

                                                let metadata = modd.paths()[0].metadata()?;
                                                #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);

                                                self.mods_mut().insert(pack_name, modd);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Then finally we process /data packs. These have the highest priority.
                if let Some(ref paths) = data_paths {
                    let paths = paths.iter()
                        .filter(|path| {
                            if let Ok(canon_path) = std::fs::canonicalize(path) {
                                let file_name = canon_path.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(String::new);
                                !vanilla_packs.contains(&canon_path) && file_name != RESERVED_PACK_NAME && file_name != RESERVED_PACK_NAME_ALTERNATIVE
                            } else {
                                false
                            }
                        })
                        .collect::<Vec<_>>();

                    let packs = paths.par_iter()
                        .map(|path| (path, Pack::read_and_merge(&[path.to_path_buf()], true, false, false)))
                        .collect::<Vec<_>>();

                    for (path, pack) in packs {
                        let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                        if let Ok(pack) = pack {
                            if pack.pfh_file_type() == PFHFileType::Mod || pack.pfh_file_type() == PFHFileType::Movie {

                                // These are not cannonicalized by default.
                                let path = std::fs::canonicalize(path)?;

                                // Check if the pack corresponds to a bin.
                                if let Some((_, modd)) = self.mods_mut().iter_mut().find(|(_, modd)| !modd.file_name().is_empty() && modd.file_name().split('/').last().unwrap() == pack_name) {
                                    if !modd.paths().contains(&path) {
                                        modd.paths_mut().insert(0, path.to_path_buf());
                                    }

                                    let metadata = modd.paths()[0].metadata()?;
                                    #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                    modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                } else {
                                    match self.mods_mut().get_mut(&pack_name) {
                                        Some(modd) => {
                                            if !modd.paths().contains(&path) {
                                                modd.paths_mut().insert(0, path.to_path_buf());
                                            }
                                            modd.set_pack_type(pack.pfh_file_type());

                                            let metadata = modd.paths()[0].metadata()?;
                                            #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        }

                                        // Same as with secondaries for legacy mods.
                                        None => {
                                            match self.mods_mut().values_mut()
                                                .filter(|modd| modd.alt_name().is_some())
                                                .find(|modd| modd.alt_name().unwrap() == pack_name) {

                                                Some(modd) => {
                                                    if !modd.paths().contains(&path) {
                                                        modd.paths_mut().insert(0, path.to_path_buf());
                                                    }
                                                    modd.set_pack_type(pack.pfh_file_type());

                                                    let metadata = modd.paths()[0].metadata()?;
                                                    #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                    modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                }

                                                None => {
                                                    let mut modd = Mod::default();
                                                    modd.set_name(pack_name.to_owned());
                                                    modd.set_id(pack_name.to_owned());
                                                    modd.set_paths(vec![path.to_path_buf()]);
                                                    modd.set_pack_type(pack.pfh_file_type());

                                                    let metadata = modd.paths()[0].metadata()?;
                                                    #[cfg(target_os = "windows")] modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                    modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);

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

        // Update the current load order to reflect any change related to mods no longer being installed or being added as new.
        let game_data_path = game.data_path(game_path)?;
        load_order.update(self, game, &game_data_path);
        load_order.save(game)?;

        // Save the GameConfig or we may lost the population.
        self.save(game)?;

        Ok(receiver)
    }
}
