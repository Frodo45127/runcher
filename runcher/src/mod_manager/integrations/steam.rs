//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//


use anyhow::Result;
use regex::Regex;
use steam_workshop_api::client::Workshop;
use steam_workshop_api::interfaces::{i_steam_remote_storage::*, i_steam_user::*, WorkshopItem};

use std::collections::HashMap;

use rpfm_ui_common::settings::setting_string;

use crate::mod_manager::mods::Mod;

lazy_static::lazy_static! {
    pub static ref REGEX_URL: Regex = Regex::new(r"(\[url=)(.*)(\])(.*)(\[/url\])").unwrap();
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub fn request_mods_data(mod_ids: &[String]) -> Result<Vec<WorkshopItem>> {
    let client = Workshop::new(None);
    get_published_file_details(&client, mod_ids)
}

pub fn request_user_names(user_ids: &[String]) -> Result<HashMap<String, String>> {
    let mut client = Workshop::new(None);
    let api_key = setting_string("steam_api_key");
    if !api_key.is_empty() {
        client.set_apikey(Some(api_key));
        get_player_names(&client, user_ids)
    } else {
        Ok(HashMap::new())
    }
}

pub fn populate_mods_with_online_data(mods: &mut HashMap<String, Mod>, workshop_items: &[WorkshopItem], last_update_date: u64) -> Result<()> {
    for workshop_item in workshop_items {
        if *workshop_item.result() == 1 {
            if let Some(modd) = mods.values_mut().filter(|modd| modd.steam_id().is_some()).find(|modd| &modd.steam_id().clone().unwrap() == workshop_item.publishedfileid()) {
                modd.set_name(workshop_item.title().clone().unwrap());
                modd.set_creator(workshop_item.creator().clone().unwrap());
                modd.set_file_name(workshop_item.filename().clone().unwrap());
                modd.set_file_size(workshop_item.file_size().unwrap());
                modd.set_file_url(workshop_item.file_url().clone().unwrap());
                modd.set_preview_url(workshop_item.preview_url().clone().unwrap());
                modd.set_description(workshop_item.description().clone().unwrap());
                modd.set_time_created(workshop_item.time_created().unwrap());
                modd.set_time_updated(workshop_item.time_updated().unwrap());

                modd.set_outdated(last_update_date > *modd.time_updated() as u64);
            }
        }
    }

    let user_ids = mods.values()
        .filter_map(|modd| if !modd.creator().is_empty() {
            Some(modd.creator().to_owned())
        } else { None }
        ).collect::<Vec<_>>();

    if let Ok(user_names) = request_user_names(&user_ids) {
        populate_mods_with_author_names(mods, &user_names);
    }

    Ok(())
}

pub fn populate_mods_with_author_names(mods: &mut HashMap<String, Mod>, user_names: &HashMap<String, String>) {
    for modd in mods.values_mut() {
        if let Some(creator_name) = user_names.get(modd.creator()) {
            modd.set_creator_name(creator_name.to_string());
        }
    }
}
