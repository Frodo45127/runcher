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
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use rpfm_lib::games::pfh_file_type::PFHFileType;

use super::v1::ModV1;

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ModV0 {

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

    // Multiple paths in case it's both in data and in a secondary folder. /data always takes priority.
    paths: Vec<PathBuf>,

    // Creator of the mod.
    creator: String,
    creator_name: String,
    file_size: u64,
    file_url: String,
    preview_url: String,
    description: String,
    time_created: usize,
    time_updated: usize,

    // Time stamp of the last time we checked. So we don't spam steam.
    last_check: u64,
}

impl From<&ModV0> for ModV1 {
    fn from(value: &ModV0) -> Self {
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
            pack_type: PFHFileType::Mod,
        }
    }
}
