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
use getset::*;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use sha256::try_digest;

use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType};
use rpfm_lib::utils::*;

use crate::settings_ui::*;

pub mod steam;
mod versions;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GameConfig {
    game_key: String,
    mods: HashMap<String, Mod>,
}

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Mod {

    // Visual name of the mod. Title if the mod is from the workshop.
    name: String,

    // Pack name of the mod.
    id: String,

    // Steam Workshop's id of this mod.
    steam_id: Option<String>,

    // If the mod is enabled or not.
    enabled: bool,

    // Category of the mod.
    category: Option<String>,

    // Pack Type of the mod. If there are multiple ones, this corresponds to the first path.
    pack_type: PFHFileType,

    // Multiple paths in case it's both in data and in a secondary folder. /data always takes priority.
    paths: Vec<PathBuf>,

    // Creator of the mod.
    creator: String,
    creator_name: String,

    // File name. If present, it's the name we need to give to the file when converting from bin to pack.
    file_name: String,
    file_size: u64,
    file_url: String,
    preview_url: String,
    description: String,
    time_created: usize,
    time_updated: usize,
    outdated: bool,

    // Time stamp of the last time we checked. So we don't spam steam.
    last_check: u64,
}

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ShareableMod {
    name: String,
    id: String,
    steam_id: Option<String>,
    hash: String
}

#[derive(Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Profile {
    id: String,
    mods: Vec<String>,
}

#[derive(Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Save {
    path: PathBuf,
    name: String,
    mods: Vec<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl GameConfig {

    pub fn load(game: &GameInfo, new_if_missing: bool) -> Result<Self> {
        let path = game_config_path()?.join(format!("game_config_{}.json", game.game_key_name()));
        if !path.is_file() && new_if_missing {
            return Ok(Self {
                game_key: game.game_key_name().to_string(),
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
        let path = game_config_path()?.join(format!("game_config_{}.json", game.game_key_name()));

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    pub fn update(game_name: &str) -> Result<()> {
        let _ = versions::v0::GameConfigV0::update(game_name);
        let _ = versions::v1::GameConfigV1::update(game_name);
        let _ = versions::v2::GameConfigV2::update(game_name);

        Ok(())
    }
}

impl Profile {

    pub fn profiles_for_game(game: &GameInfo) -> Result<HashMap<String, Self>> {
        let mut profiles = HashMap::new();
        let path = profiles_path()?;
        let file_name_start = format!("profile_{}_", game.game_key_name());

        let files = files_from_subdir(&path, false)?;
        for file in files {
            let file_name = file.file_name().unwrap().to_string_lossy();
            if file_name.starts_with(&file_name_start) && file_name.ends_with(".json") {
                let file_name_no_end = file.file_stem().unwrap().to_string_lossy().strip_prefix(&file_name_start).unwrap().to_string();
                let profile = Self::load(game, &file_name_no_end, false)?;
                profiles.insert(file_name_no_end, profile);
            }
        }

        Ok(profiles)
    }

    pub fn load(game: &GameInfo, profile: &str, new_if_missing: bool) -> Result<Self> {
        let path = profiles_path()?.join(format!("profile_{}_{}.json", game.game_key_name(), profile));
        if !path.is_file() && new_if_missing {
            return Ok(Self {
                id: profile.to_string(),
                ..Default::default()
            });
        }

        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;

        let profile: Self = serde_json::from_slice(&data)?;
        Ok(profile)
    }

    pub fn save(&mut self, game: &GameInfo, profile: &str) -> Result<()> {
        let path = profiles_path()?.join(format!("profile_{}_{}.json", game.game_key_name(), profile));

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }
}

impl From<&Mod> for ShareableMod {

    fn from(value: &Mod) -> Self {
        let hash = try_digest(value.paths()[0].as_path()).unwrap();
        Self {
            name: value.name().to_owned(),
            id: value.id().to_owned(),
            steam_id: value.steam_id().to_owned(),
            hash,
        }
    }
}
