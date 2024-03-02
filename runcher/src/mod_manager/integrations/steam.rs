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
use interprocess::local_socket::LocalSocketListener;
use regex::Regex;
use serde::Deserialize;
use steam_workshop_api::client::Workshop;
use steam_workshop_api::interfaces::i_steam_user::*;

use std::collections::HashMap;
use std::io::Read;
use std::process::Command;
#[cfg(target_os = "windows")]use std::os::windows::process::CommandExt;

use rpfm_lib::games::GameInfo;
use rpfm_ui_common::settings::{setting_path, setting_string};

use crate::mod_manager::mods::Mod;

lazy_static::lazy_static! {
    pub static ref REGEX_URL: Regex = Regex::new(r"(\[url=)(.*)(\])(.*)(\[/url\])").unwrap();

    static ref WORKSHOPPER_PATH: String = if cfg!(debug_assertions) {
        format!(".\\target\\debug\\{}", WORKSHOPPER_EXE)
    } else {
        WORKSHOPPER_EXE.to_string()
    };
}

const WORKSHOPPER_EXE: &str = "workshopper.exe";

const IPC_NAME_GET_PUBLISHED_FILE_DETAILS: &str = "runcher_get_published_file_details";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Clone, Deserialize)]
pub struct QueryResultDerive {
    pub published_file_id: u64,
    pub creator_app_id: Option<u32>,
    pub consumer_app_id: Option<u32>,
    pub title: String,
    pub description: String,
    pub owner: u64,
    pub time_created: u32,
    pub time_updated: u32,
    pub time_added_to_user_list: u32,
    pub visibility: PublishedFileVisibilityDerive,
    pub banned: bool,
    pub accepted_for_use: bool,
    pub tags: Vec<String>,
    pub tags_truncated: bool,
    pub file_name: String,
    pub file_type: FileTypeDerive,
    pub file_size: u32,
    pub url: String,
    pub num_upvotes: u32,
    pub num_downvotes: u32,
    pub score: f32,
    pub num_children: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum PublishedFileVisibilityDerive {
    Public,
    FriendsOnly,
    Private,
    Unlisted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum FileTypeDerive {
    Community,
    Microtransaction,
    Collection,
    Art,
    Video,
    Screenshot,
    Game,
    Software,
    Concept,
    WebGuide,
    IntegratedGuide,
    Merch,
    ControllerBinding,
    SteamworksAccessInvite,
    SteamVideo,
    GameManagedItem,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub fn request_mods_data(game: &GameInfo, mod_ids: &[String]) -> Result<Vec<Mod>> {
    let game_path = setting_path(game.key());
    let steam_id = game.steam_id(&game_path)? as u32;
    let published_file_ids = mod_ids.join(",");

    let mut command = Command::new("cmd");
    command.arg("/C");
    command.arg(&*WORKSHOPPER_PATH);

    command.arg("get-published-file-details");
    command.arg("-s");
    command.arg(steam_id.to_string());
    command.arg("-p");
    command.arg(published_file_ids);

    // This is for creating the terminal window. Without it, the entire process runs in the background and there's no feedback on when it's done.
    #[cfg(target_os = "windows")] if cfg!(debug_assertions) {
        command.creation_flags(0x00000008);
    }

    command.spawn()?;

    let server = LocalSocketListener::bind(IPC_NAME_GET_PUBLISHED_FILE_DETAILS)?;
    let mut stream = server.accept()?;

    let mut message = String::new();
    stream.read_to_string(&mut message)?;

    let deserialized_data: Vec<QueryResultDerive> = serde_json::from_str(&message)?;

    let mut mods = vec![];
    for workshop_item in &deserialized_data {
        let mut modd = Mod::default();
        modd.set_steam_id(Some(workshop_item.published_file_id.to_string()));

        modd.set_name(workshop_item.title.to_owned());
        modd.set_creator(workshop_item.owner.to_string());
        modd.set_file_name(workshop_item.file_name.to_owned());
        modd.set_file_size(workshop_item.file_size as u64);
        modd.set_file_url(workshop_item.url.to_owned());
        //modd.set_preview_url(workshop_item.preview_url.clone().unwrap());
        modd.set_description(workshop_item.description.to_owned());
        modd.set_time_created(workshop_item.time_created as usize);
        modd.set_time_updated(workshop_item.time_updated as usize);

        mods.push(modd);
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

    let mut command = Command::new("cmd");
    command.arg("/C");
    command.arg(&*WORKSHOPPER_PATH);

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