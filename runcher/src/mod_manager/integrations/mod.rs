//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Online integrations. The intention is so this module acts as a common abstraction of specific integrations.
//!
//! For now we only support steam workshop, so all calls are redirected to the steam module.

use anyhow::Result;
use serde::Deserialize;

use std::collections::HashMap;

use rpfm_lib::games::GameInfo;

use crate::mod_manager::mods::Mod;

mod steam;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum PublishedFileVisibilityDerive {
    Public,
    FriendsOnly,
    #[default] Private,
    Unlisted,
}

#[derive(Debug, Clone, Default)]
pub struct PreUploadInfo {
    pub published_file_id: u64,
    pub title: String,
    pub description: String,
    pub visibility: PublishedFileVisibilityDerive,
    pub tags: Vec<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub fn request_mods_data(game: &GameInfo, mod_ids: &[String]) -> Result<Vec<Mod>> {
    steam::request_mods_data(game, mod_ids)
}

pub fn request_pre_upload_info(game: &GameInfo, mod_id: &str, owner_id: &str) -> Result<PreUploadInfo> {
    steam::request_pre_upload_info(game, mod_id, owner_id)
}

pub fn populate_mods_with_online_data(mods: &mut HashMap<String, Mod>, workshop_items: &[Mod]) -> Result<()> {
    steam::populate_mods_with_online_data(mods, workshop_items)
}

pub fn upload_mod_to_workshop(game: &GameInfo, modd: &Mod, title: &str, description: &str, tags: &[String], changelog: &str, visibility: &Option<u32>) -> Result<()> {
    steam::upload_mod_to_workshop(game, modd, title, description, tags, changelog, visibility)
}

pub fn launch_game(game: &GameInfo, command_to_pass: &str) -> Result<()> {
    steam::launch_game(game, command_to_pass)
}

pub fn download_subscribed_mods(game: &GameInfo) -> Result<()> {
    steam::download_subscribed_mods(game)
}
