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
use serde_json::to_string_pretty;

use std::collections::{HashMap, BTreeMap};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};

use rpfm_lib::games::{GameInfo, supported_games::SupportedGames};

use crate::game_config_path;
use crate::mod_manager::game_config::DEFAULT_CATEGORY;
use crate::mod_manager::mods::{Mod as ModV4, versions::v3::ModV3};

use super::GameConfigV4;

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GameConfigV3 {
    pub game_key: String,
    pub mods: HashMap<String, ModV3>,
}

impl GameConfigV3 {
    pub fn update(game_name: &str) -> Result<()> {
        let games = SupportedGames::default();
        if let Some(game_info) = games.game(game_name) {
            if let Ok(config) = Self::load(game_info, false) {
                let mut config_new = GameConfigV4::from(&config);
                dbg!(1);
                config_new.save(game_info)?;
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

impl From<&GameConfigV3> for GameConfigV4 {
    fn from(value: &GameConfigV3) -> Self {

        // Migrate to the new categories list.
        let mut categories: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut categories_order = vec![];
        for (key, modd) in &value.mods {
            let category = modd.category.clone().unwrap_or_else(|| DEFAULT_CATEGORY.to_string());
            match categories.get_mut(&category) {
                Some(mods) => mods.push(key.to_owned()),
                None => { categories.insert(category.to_string(), vec![key.to_owned()]); },
            }

            if !categories_order.contains(&category) && category != DEFAULT_CATEGORY {
                categories_order.push(category);
            }
        }

        // Make sure the default category is always last when updating files.
        categories_order.push(DEFAULT_CATEGORY.to_owned());

        Self {
            game_key: value.game_key.to_owned(),
            mods: value.mods.iter().map(|(key, value)| (key.to_owned(), ModV4::from(value))).collect::<HashMap<_, _>>(),
            categories,
            categories_order,
        }
    }
}
