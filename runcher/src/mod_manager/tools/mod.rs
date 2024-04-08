//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;
use getset::*;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use rpfm_ui_common::settings::config_path;

const TOOLS_FILE: &str = "tools.json";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Tools {
    tools: Vec<Tool>,
}

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Tool {
    name: String,
    path: PathBuf,
    games: Vec<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Tools {

    pub fn load() -> Result<Self> {
        let path = config_path()?.join(TOOLS_FILE);

        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;

        // Cleanup the loaded order to make sure it's not including not installed packs, or new packs.
        let order: Self = serde_json::from_slice(&data)?;

        Ok(order)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?.join(TOOLS_FILE);

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }
}
