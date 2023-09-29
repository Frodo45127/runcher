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

use crate::mod_manager::mods::Mod;

mod steam;

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

#[allow(dead_code)]
pub fn populate_mods(mods: &mut HashMap<String, Mod>, mod_ids: &[String], last_update_date: u64) -> Result<()> {
    steam::populate_mods(mods, mod_ids, last_update_date)
}

#[allow(dead_code)]
pub fn populate_user_names(mods: &mut HashMap<String, Mod>, user_ids: &[String]) -> Result<()> {
    steam::populate_user_names(mods, user_ids)
}
