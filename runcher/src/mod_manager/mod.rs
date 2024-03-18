//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing the centralized code for mod and load order management.
//!
//! Here are also generic functions for mod managing.

use anyhow::{anyhow, Result};

use std::fs::DirBuilder;
use std::path::PathBuf;

use rpfm_lib::utils::files_from_subdir;
use rpfm_ui_common::settings::setting_string;

use crate::SUPPORTED_GAMES;

pub mod game_config;
pub mod integrations;
pub mod load_order;
pub mod mods;
pub mod profiles;
pub mod saves;
pub mod tools;

pub fn secondary_mods_path(game: &str) -> Result<PathBuf> {
    match SUPPORTED_GAMES.game(game) {
        Some(game_info) => if game_info.raw_db_version() < &1 {
            return Err(anyhow!("This game ({}) doesn't support secondary mod folders.", game))
        }
        None => return Err(anyhow!("What kind of game is {}?", game)),
    }

    let base_path_str = setting_string("secondary_mods_path");
    if base_path_str.is_empty() {
        return Err(anyhow!("Secondary Mods Path not set."))
    }

    // Canonicalization is required due to some issues with the game not loading not properly formatted paths.
    let path = std::fs::canonicalize(PathBuf::from(base_path_str))?;
    let game_path = path.join(game);

    if !path.is_dir() {
        DirBuilder::new().recursive(true).create(&path)?;
    }

    Ok(path)
}

pub fn secondary_mods_packs_paths(game: &str) -> Option<Vec<PathBuf>> {
    let path = secondary_mods_path(game).ok()?;
    let mut paths = vec![];

    for path in files_from_subdir(&path, true).ok()?.iter() {
        match path.extension() {
            Some(extension) => if extension == "pack" || extension == "bin" { paths.push(path.to_path_buf()); }
            None => continue,
        }
    }

    paths.sort();

    Some(paths)
}
