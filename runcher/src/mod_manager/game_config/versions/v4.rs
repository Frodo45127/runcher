//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::collections::{HashMap, BTreeMap};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};

use rpfm_lib::games::{GameInfo, supported_games::SupportedGames};

use crate::game_config_path;
use crate::mod_manager::mods::{Mod as ModV5, versions::v4::ModV4};

use super::GameConfigV5;

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GameConfigV4 {
    pub game_key: String,
    pub mods: HashMap<String, ModV4>,
    pub categories: BTreeMap<String, Vec<String>>,
    pub categories_order: Vec<String>,
}

impl GameConfigV4 {
    pub fn update(game_name: &str) -> Result<()> {
        let games = SupportedGames::default();
        if let Some(game_info) = games.game(game_name) {
            if let Ok(config) = Self::load(game_info, false) {

                // Check that it fails with v4, because v4 files for some reason are readable with v3.
                if GameConfigV5::load(game_info, false).is_err() {
                    let mut config_new = GameConfigV5::from(&config);
                    config_new.save(game_info)?;
                }
            }
        }

        Ok(())
    }

    pub fn load(game: &GameInfo, new_if_missing: bool) -> Result<Self> {
        let path = game_config_path()?.join(format!("game_config_{}.json", game.key()));
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
        let path = game_config_path()?.join(format!("game_config_{}.json", game.key()));

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }
}

impl From<&GameConfigV4> for GameConfigV5 {
    fn from(value: &GameConfigV4) -> Self {
        Self {
            game_key: value.game_key.to_owned(),
            mods: value.mods.iter().map(|(key, value)| (key.to_owned(), ModV5::from(value))).collect::<HashMap<_, _>>(),
            categories: value.categories.clone(),
            categories_order: value.categories_order.clone(),
        }
    }
}
