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
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Save {
    path: PathBuf,
    name: String,
    mods: Vec<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//
