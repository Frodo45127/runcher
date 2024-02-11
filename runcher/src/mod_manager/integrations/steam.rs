//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//


use anyhow::{anyhow, Result};
use regex::Regex;
use steam_workshop_api::client::Workshop;
use steam_workshop_api::interfaces::{i_steam_remote_storage::*, i_steam_user::*};

use std::collections::HashMap;
use std::process::Command;
#[cfg(target_os = "windows")]use std::os::windows::process::CommandExt;

use rpfm_lib::games::GameInfo;
use rpfm_ui_common::settings::{setting_path, setting_string};

use crate::mod_manager::mods::Mod;

lazy_static::lazy_static! {
    pub static ref REGEX_URL: Regex = Regex::new(r"(\[url=)(.*)(\])(.*)(\[/url\])").unwrap();
}

const WORKSHOPPER_EXE: &str = "workshopper.exe";

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub fn request_mods_data(mod_ids: &[String]) -> Result<Vec<Mod>> {
    let client = Workshop::new(None);
    let workshop_items = get_published_file_details(&client, mod_ids)?;

    let mut mods = vec![];

    // Note: this processes mods retrieved through the Web API. Mods owned by the user but hidden are not here.
    for workshop_item in workshop_items {
        if *workshop_item.result() == 1 {
            let mut modd = Mod::default();
            modd.set_steam_id(Some(workshop_item.publishedfileid().to_owned()));

            modd.set_name(workshop_item.title().clone().unwrap());
            modd.set_creator(workshop_item.creator().clone().unwrap());
            modd.set_file_name(workshop_item.filename().clone().unwrap());
            modd.set_file_size(workshop_item.file_size().unwrap());
            modd.set_file_url(workshop_item.file_url().clone().unwrap());
            modd.set_preview_url(workshop_item.preview_url().clone().unwrap());
            modd.set_description(workshop_item.description().clone().unwrap());
            modd.set_time_created(workshop_item.time_created().unwrap());
            modd.set_time_updated(workshop_item.time_updated().unwrap());

            mods.push(modd);
        }
    }

    Ok(mods)
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

pub fn populate_mods_with_online_data(mods: &mut HashMap<String, Mod>, workshop_items: &[Mod], last_update_date: u64) -> Result<()> {
    for workshop_item in workshop_items {
        if let Some(modd) = mods.values_mut()
            .filter(|modd| modd.steam_id().is_some())
            .find(|modd| modd.steam_id() == workshop_item.steam_id()) {

            modd.set_name(workshop_item.name().to_string());
            modd.set_creator(workshop_item.creator().to_string());
            modd.set_file_name(workshop_item.file_name().to_string());
            modd.set_file_size(*workshop_item.file_size());
            modd.set_file_url(workshop_item.file_url().to_string());
            modd.set_preview_url(workshop_item.preview_url().to_string());
            modd.set_description(workshop_item.description().to_string());
            modd.set_time_created(*workshop_item.time_created());
            modd.set_time_updated(*workshop_item.time_updated());

            modd.set_outdated(last_update_date > *modd.time_updated() as u64);
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

/// This function uploads a mod to the workshop through workshopper.
///
/// If the mod doesn't yet exists in the workshop, it creates it. If it already exists, it updates it.
pub fn upload_mod_to_workshop(game: &GameInfo, modd: &Mod, title: &str, description: &str, tags: &[String], changelog: &str) -> Result<()> {
    let game_path = setting_path(game.key());
    let steam_id = game.steam_id(&game_path)? as u32;

    let pack_path = if modd.paths().is_empty() {
        return Err(anyhow!("Mod Path not found."));
    } else {
        &modd.paths()[0]
    };

    let exe_path = if cfg!(debug_assertions) {
        format!(".\\target\\debug\\{}", WORKSHOPPER_EXE)
    } else {
        WORKSHOPPER_EXE.to_string()
    };

    let mut command = Command::new("cmd");
    command.arg("/C");
    command.arg(exe_path);

    // If we have a published_file_id, it means this file exists in the workshop.
    //
    // So, instead of uploading, we just update it.
    match modd.steam_id() {
        Some(published_file_id) => {
            command.arg("update");
            command.arg("--published_file_id");
            command.arg(published_file_id);
        }
        None => {
            command.arg("upload");
        }
    }

    command.arg("-s");
    command.arg(steam_id.to_string());
    command.arg("-p");
    command.arg(pack_path.to_string_lossy().to_string());
    command.arg("-t");
    command.arg(title);

    if !description.is_empty() {
        command.arg("-d");
        command.arg(description);
    }

    command.arg("--tags");
    command.arg(tags.join(","));

    if !changelog.is_empty() {
        command.arg("-c");
        command.arg(changelog);
    }

    // This is for creating the terminal window. Without it, the entire process runs in the background and there's no feedback on when it's done.
    #[cfg(target_os = "windows")]command.creation_flags(0x00000008);

    command.spawn()?;

    Ok(())
}
