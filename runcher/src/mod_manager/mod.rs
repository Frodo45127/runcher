//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use std::fs::{DirBuilder, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use rpfm_lib::games::GameInfo;
use rpfm_lib::utils::{files_from_subdir, path_to_absolute_path, path_to_absolute_string};

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::settings::*;

use crate::SUPPORTED_GAMES;

use self::game_config::GameConfig;

pub mod game_config;
pub mod integrations;
pub mod load_order;
pub mod mods;
pub mod profiles;
pub mod saves;

pub const SECONDARY_FOLDER_NAME: &str = "masks";

pub fn copy_to_secondary(game: &GameInfo, game_config: &GameConfig, mod_ids: &[String]) -> Result<Vec<String>> {
    let mut mods_failed = vec![];

    let game_path = setting_path(game.key());
    let secondary_path = secondary_mods_path(game.key())?;
    let content_path = path_to_absolute_path(&game.content_path(&game_path)?, true);
    let secondary_path_str = path_to_absolute_string(&secondary_path);
    let content_path_str = path_to_absolute_string(&content_path);

    for mod_id in mod_ids {
        if let Some(modd) = game_config.mods().get(mod_id) {

            // Apply only to mods on content, or both on content and secondary.
            if modd.paths().len() <= 2 {
                let decannon_paths = modd.paths()
                    .iter()
                    .map(|path| path_to_absolute_string(path))
                    .collect::<Vec<_>>();

                // If there's only one path, check if it's in content.
                if decannon_paths.len() == 1 && decannon_paths[0].starts_with(&content_path_str) {
                    let new_path = secondary_path.join(modd.paths()[0].file_name().unwrap());
                    if std::fs::copy(&modd.paths()[0], new_path).is_err() {
                        mods_failed.push(modd.id().to_string());
                    }

                    // Copy the png too.
                    else {

                        let mut old_image_path = PathBuf::from(&decannon_paths[0]);
                        old_image_path.set_extension("png");

                        let mut new_image_path = secondary_path.join(modd.paths()[0].file_name().unwrap());
                        new_image_path.set_extension("png");

                        let _ = std::fs::copy(&old_image_path, &new_image_path);
                    }
                }

                // If it's a file in content and secondary, allow to copy it to update the secondary one.
                else if decannon_paths.len() == 2 && decannon_paths[0].starts_with(&secondary_path_str) && decannon_paths[1].starts_with(&content_path_str) {
                    if std::fs::copy(&modd.paths()[1], &modd.paths()[0]).is_err() {
                        mods_failed.push(modd.id().to_string());
                    }

                    // Copy the png too.
                    else {
                        let mut old_image_path = PathBuf::from(&decannon_paths[1]);
                        old_image_path.set_extension("png");

                        let mut new_image_path = PathBuf::from(&decannon_paths[0]);
                        new_image_path.set_extension("png");

                        let _ = std::fs::copy(&old_image_path, &new_image_path);
                    }
                }

                // Any other case is not supported.
                else {
                    mods_failed.push(modd.id().to_string());
                }
            }
        }
    }

    Ok(mods_failed)
}

pub fn move_to_secondary(game: &GameInfo, game_config: &GameConfig, mod_ids: &[String]) -> Result<Vec<String>> {
    let mut mods_failed = vec![];

    let game_path = setting_path(game.key());
    let secondary_path = secondary_mods_path(game.key())?;
    let data_path = game.data_path(&game_path)?;
    let data_path_str = path_to_absolute_string(&data_path);

    for mod_id in mod_ids {
        if let Some(modd) = game_config.mods().get(mod_id) {

            // Apply only to mods on content, or both on content and secondary.
            let decannon_paths = modd.paths()
                .iter()
                .map(|path| path_to_absolute_string(path))
                .collect::<Vec<_>>();

            // If the first path is /data, proceed. If not, we cannot move this mod.
            if decannon_paths[0].starts_with(&data_path_str) {
                let new_path = secondary_path.join(modd.paths()[0].file_name().unwrap());
                if std::fs::copy(&modd.paths()[0], new_path).is_err() {
                    mods_failed.push(modd.id().to_string());
                }

                // Move the png too, and delete the originals if it worked.
                else {

                    let mut old_image_path = PathBuf::from(&decannon_paths[0]);
                    old_image_path.set_extension("png");

                    let mut new_image_path = secondary_path.join(modd.paths()[0].file_name().unwrap());
                    new_image_path.set_extension("png");

                    if std::fs::copy(&old_image_path, &new_image_path).is_ok() {
                        let _ = std::fs::remove_file(&modd.paths()[0]);
                        let _ = std::fs::remove_file(&old_image_path);
                    }
                }
            }

            // Any other case is not supported.
            else {
                mods_failed.push(modd.id().to_string());
            }
        }
    }

    Ok(mods_failed)
}

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

    if !game_path.is_dir() {
        DirBuilder::new().recursive(true).create(&game_path)?;
    }

    Ok(game_path)
}

pub fn secondary_mods_packs_paths(game: &str) -> Option<Vec<PathBuf>> {
    let path = secondary_mods_path(game).ok()?;
    let mut paths = vec![];

    for path in files_from_subdir(&path, false).ok()?.iter() {
        match path.extension() {
            Some(extension) => if extension == "pack" || extension == "bin" { paths.push(path.to_path_buf()); }
            None => continue,
        }
    }

    paths.sort();

    Some(paths)
}

pub unsafe fn icon_data(icon_file_name: &str) -> Result<Vec<u8>> {
    let icon_path = format!("{}/icons/{icon_file_name}", ASSETS_PATH.to_string_lossy());
    let mut icon_file = File::open(icon_path)?;
    let mut data = Vec::with_capacity(icon_file.metadata()?.len() as usize);

    icon_file.read_to_end(&mut data)?;
    icon_file.flush()?;

    Ok(data)
}
