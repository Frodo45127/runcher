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
use rpfm_lib::games::pfh_file_type::PFHFileType;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::{BufReader, Read};

use rpfm_lib::games::GameInfo;
use rpfm_lib::utils::files_from_subdir;

use crate::mod_manager::{game_config::GameConfig, load_order::LoadOrder};
use crate::profiles_path;

use super::ProfileV1;

const PROFILE_FILE_NAME_START: &str = "profile_";
const PROFILE_FILE_NAME_END: &str = ".json";

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ProfileV0 {
    id: String,
    mods: Vec<String>,
}

impl ProfileV0 {
    pub fn update(game_config: &GameConfig, game: &GameInfo) -> Result<()> {
        let path = profiles_path()?;
        let file_name_start = format!("{PROFILE_FILE_NAME_START}{}_", game.key());

        let files = files_from_subdir(&path, false)?;
        for file in files {
            let file_name = file.file_name().unwrap().to_string_lossy();
            if file_name.starts_with(&file_name_start) && file_name.ends_with(PROFILE_FILE_NAME_END) {
                let file_name_no_end = file.file_stem().unwrap().to_string_lossy().strip_prefix(&file_name_start).unwrap().to_string();
                if let Ok(profile) = Self::load(game, &file_name_no_end, false) {

                    let mut profile = ProfileV1::from(&profile);
                    profile.set_game(game.key().to_string());

                    let movies = profile.load_order().mods()
                        .iter()
                        .filter_map(|mod_id| game_config.mods().get(mod_id))
                        .filter(|modd| modd.pack_type() == &PFHFileType::Movie)
                        .map(|modd| modd.id().to_owned())
                        .collect::<Vec<_>>();

                    profile.load_order_mut().mods_mut().retain(|mod_id| !movies.contains(&mod_id));
                    *profile.load_order_mut().movies_mut() = movies;

                    let profile_name = profile.id.to_owned();
                    profile.save(game, &profile_name)?;
                }
            }
        }

        Ok(())
    }

    pub fn load(game: &GameInfo, profile: &str, new_if_missing: bool) -> Result<Self> {
        let path = profiles_path()?.join(format!("{PROFILE_FILE_NAME_START}{}_{}{PROFILE_FILE_NAME_END}", game.key(), profile));
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
}

impl From<&ProfileV0> for ProfileV1 {
    fn from(value: &ProfileV0) -> Self {
        let mut load_order = LoadOrder::default();
        *load_order.mods_mut() = value.mods().to_vec();

        Self {
            id: value.id().to_string(),
            game: String::new(),        // To be filled after the from.
            load_order,                 // Movies need to be removed from this later.
        }
    }
}
