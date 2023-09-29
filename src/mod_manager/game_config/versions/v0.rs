//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};

use rpfm_lib::games::{GameInfo, supported_games::SupportedGames};

use crate::game_config_path;
use crate::mod_manager::mods::versions::{v0::ModV0, v1::ModV1};

use super::v1::GameConfigV1;

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GameConfigV0 {
    game_key: String,
    mods: HashMap<String, ModV0>,
}

impl GameConfigV0 {
    pub fn update(game_name: &str) -> Result<()> {
        let games = SupportedGames::default();
        if let Some(game_info) = games.game(game_name) {
            let config = Self::load(game_info, false)?;

            let mut config_new = GameConfigV1::from(&config);
            config_new.save(game_info)?;
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
}

impl From<&GameConfigV0> for GameConfigV1 {
    fn from(value: &GameConfigV0) -> Self {
        Self {
            game_key: value.game_key.to_owned(),
            mods: value.mods.iter().map(|(key, value)| (key.to_owned(), ModV1::from(value))).collect::<HashMap<_, _>>(),
        }
    }
}
