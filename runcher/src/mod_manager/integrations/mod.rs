//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use std::collections::HashMap;

use rpfm_lib::games::GameInfo;

use crate::mod_manager::mods::Mod;

mod steam;

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub fn request_mods_data(game: &GameInfo, mod_ids: &[String]) -> Result<Vec<Mod>> {
    steam::request_mods_data(game, mod_ids)
}

pub fn populate_mods_with_online_data(mods: &mut HashMap<String, Mod>, workshop_items: &[Mod], last_update_date: u64) -> Result<()> {
    steam::populate_mods_with_online_data(mods, workshop_items, last_update_date)
}

pub fn upload_mod_to_workshop(game: &GameInfo, modd: &Mod, title: &str, description: &str, tags: &[String], changelog: &str) -> Result<()> {
    steam::upload_mod_to_workshop(game, modd, title, description, tags, changelog)
}
