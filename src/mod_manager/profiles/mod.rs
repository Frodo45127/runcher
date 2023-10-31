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
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};

use rpfm_lib::games::GameInfo;
use rpfm_lib::utils::*;

use crate::mod_manager::game_config::GameConfig;
use crate::settings_ui::*;

use super::load_order::LoadOrder;

mod versions;

const FILE_NAME_START: &str = "profile_";
const FILE_NAME_END: &str = ".json";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Profile {

    // Id/Name of the profile. Must be unique for the game.
    id: String,

    // Game this profile belongs to.
    game: String,

    // Load order of this profile.
    load_order: LoadOrder,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Profile {

    pub fn profiles_for_game(game: &GameInfo) -> Result<HashMap<String, Self>> {
        let mut profiles = HashMap::new();
        let path = profiles_path()?;
        let file_name_start = format!("{FILE_NAME_START}{}_", game.key());

        let files = files_from_subdir(&path, false)?;
        for file in files {
            let file_name = file.file_name().unwrap().to_string_lossy();
            if file_name.starts_with(&file_name_start) && file_name.ends_with(FILE_NAME_END) {
                let file_name_no_end = file.file_stem().unwrap().to_string_lossy().strip_prefix(&file_name_start).unwrap().to_string();
                let profile = Self::load(game, &file_name_no_end, false)?;
                profiles.insert(file_name_no_end, profile);
            }
        }

        Ok(profiles)
    }

    pub fn load(game: &GameInfo, profile: &str, new_if_missing: bool) -> Result<Self> {
        let path = profiles_path()?.join(format!("{FILE_NAME_START}{}_{}{FILE_NAME_END}", game.key(), profile));
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
        let path = profiles_path()?.join(format!("{FILE_NAME_START}{}_{}{FILE_NAME_END}", game.key(), profile));

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update(game_config: &GameConfig, game_info: &GameInfo) -> Result<()> {
        let _ = versions::v0::ProfileV0::update(game_config, game_info);

        Ok(())
    }
}
