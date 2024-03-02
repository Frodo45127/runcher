//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use super::v2::ModV2;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModV1 {
    pub name: String,
    pub id: String,
    pub steam_id: Option<String>,
    pub enabled: bool,
    pub category: Option<String>,
    pub pack_type: PFHFileType,
    pub paths: Vec<PathBuf>,
    pub creator: String,
    pub creator_name: String,
    pub file_size: u64,
    pub file_url: String,
    pub preview_url: String,
    pub description: String,
    pub time_created: usize,
    pub time_updated: usize,
    pub last_check: u64,
}

impl From<&ModV1> for ModV2 {
    fn from(value: &ModV1) -> Self {
        Self {
            name: value.name.to_owned(),
            id: value.id.to_owned(),
            steam_id: value.steam_id.to_owned(),
            enabled: value.enabled,
            category: value.category.to_owned(),
            paths: value.paths.to_owned(),
            creator: value.creator.to_owned(),
            creator_name: value.creator_name.to_owned(),
            file_size: value.file_size,
            file_url: value.file_url.to_owned(),
            preview_url: value.preview_url.to_owned(),
            description: value.description.to_owned(),
            time_created: value.time_created,
            time_updated: value.time_updated,
            last_check: value.last_check,
            outdated: false,
            pack_type: PFHFileType::Mod,
        }
    }
}
