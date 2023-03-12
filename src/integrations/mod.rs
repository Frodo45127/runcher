//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;

use std::path::PathBuf;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct Mod {
    name: String,
    category: String,
    pack: PathBuf,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Default for Mod {
    fn default() -> Self {
        Self {
            name: String::default(),
            category: "Unknown".to_owned(),
            pack: PathBuf::new(),
        }
    }
}
