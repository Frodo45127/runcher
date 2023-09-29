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
use sha256::try_digest;

use std::path::PathBuf;

use rpfm_lib::games::pfh_file_type::PFHFileType;

pub mod versions;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

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

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

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
