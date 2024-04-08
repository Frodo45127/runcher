//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use rpfm_lib::games::pfh_file_type::PFHFileType;

use super::ModV5;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModV4 {
    pub name: String,
    pub id: String,
    pub steam_id: Option<String>,
    pub enabled: bool,
    pub pack_type: PFHFileType,
    pub paths: Vec<PathBuf>,
    pub creator: String,
    pub creator_name: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_url: String,
    pub preview_url: String,
    pub description: String,
    pub time_created: usize,
    pub time_updated: usize,
    pub outdated: bool,
    pub last_check: u64,
}

impl From<&ModV4> for ModV5 {
    fn from(value: &ModV4) -> Self {
        Self {
            name: value.name.to_owned(),
            id: value.id.to_owned(),
            steam_id: value.steam_id.to_owned(),
            enabled: value.enabled,
            paths: value.paths.to_owned(),
            creator: value.creator.to_owned(),
            creator_name: value.creator_name.to_owned(),
            file_name: String::new(),
            file_size: value.file_size,
            description: value.description.to_owned(),
            time_created: value.time_created,
            time_updated: value.time_updated,
            pack_type: PFHFileType::Mod,
        }
    }
}
