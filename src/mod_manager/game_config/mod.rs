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

use crate::mod_manager::mods::Mod;
use crate::settings_ui::*;

mod versions;

const GAME_CONFIG_FILE_NAME_START: &str = "game_config_";
const GAME_CONFIG_FILE_NAME_END: &str = ".json";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GameConfig {
    game_key: String,
    mods: HashMap<String, Mod>,

    // TODO: Move the load order here, so it's always available and up to date.
    // TODO2: Split categories out from mods into a field here, so we can better manage them.
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

    pub fn update(game_name: &str) -> Result<()> {
        let _ = versions::v0::GameConfigV0::update(game_name);
        let _ = versions::v1::GameConfigV1::update(game_name);
        let _ = versions::v2::GameConfigV2::update(game_name);

        Ok(())
    }
}
