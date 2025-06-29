//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::{anyhow, Result};
use base64::prelude::*;
use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions};
use serde::Deserialize;
use steam_workshop_api::{client::Workshop, interfaces::i_steam_user::*};

use std::cell::LazyCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(target_os = "windows")]use std::os::windows::process::CommandExt;

use rpfm_lib::files::{EncodeableExtraData, pack::Pack};
use rpfm_lib::games::GameInfo;
use rpfm_lib::utils::path_to_absolute_string;

use rpfm_ui_common::settings::{setting_path, setting_string};

use crate::mod_manager::mods::Mod;

#[cfg(target_os = "windows")]use super::{CREATE_NEW_CONSOLE, CREATE_NO_WINDOW, DETACHED_PROCESS};
use super::{PreUploadInfo, PublishedFileVisibilityDerive};

const WORKSHOPPER_PATH: LazyCell<String> = LazyCell::new(|| {
    let base_path = std::env::current_dir().unwrap();
    let base_path = base_path.display();
    if cfg!(debug_assertions) {
        format!("{}/target/debug/{}", base_path, WORKSHOPPER_EXE)
    } else {
        format!("{}/{}", base_path, WORKSHOPPER_EXE)
    }
});

const WORKSHOPPER_EXE: &str = "workshopper.exe";

const BAT_UPLOAD_TO_WORKSHOP: &str = "upload-to-workshop.bat";
const BAT_LAUNCH_GAME: &str = "launch-game.bat";
const BAT_USER_ID: &str = "user-id.bat";
const BAT_DOWNLOAD_SUBSCRIBED_ITEMS: &str = "download-subscribed-items.bat";
const BAT_GET_PUBLISHED_FILE_DETAILS: &str = "get-published-file-details.bat";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Clone, Deserialize)]
pub struct QueryResultDerive {
    pub published_file_id: u64,
    pub title: String,
    pub description: String,
    pub owner: u64,
    pub time_created: u32,
    pub time_updated: u32,
    pub visibility: PublishedFileVisibilityDerive,
    pub tags: Vec<String>,
    pub file_name: String,
    pub file_size: u32,
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

impl From<&QueryResultDerive> for PreUploadInfo {
    fn from(value: &QueryResultDerive) -> Self {
        Self {
            published_file_id: value.published_file_id.clone(),
            title: value.title.clone(),
            description: value.description.clone(),
            visibility: value.visibility.clone(),
            tags: value.tags.to_vec(),
        }
    }
}

pub fn request_pre_upload_info(game: &GameInfo, mod_id: &str) -> Result<PreUploadInfo> {
    let workshop_items = request_mods_data_raw(game, &[mod_id.to_owned()])?;
    if workshop_items.is_empty() {
        return Err(anyhow!("Mod with SteamId {} not found in the Workshop.", mod_id));
    }

    // If we're not the author, do not even let us upload it.
    //let steam_user_id = user_id(game)?.to_string();
    //if steam_user_id.is_empty() || owner_id != steam_user_id {
    //    return Err(anyhow!("You're not the original uploader of this mod, or steam hasn't been detected on your system."));
    //}

    let workshop_item = workshop_items.first().unwrap();
    let data = PreUploadInfo::from(workshop_item);

    Ok(data)
}

pub fn request_mods_data(game: &GameInfo, mod_ids: &[String]) -> Result<Vec<Mod>> {

    // Do not call the cmd if there are no mods.
    if mod_ids.is_empty() {
        return Ok(vec![])
    }

    let workshop_items = request_mods_data_raw(game, mod_ids)?;

    let mut mods = vec![];
    for workshop_item in &workshop_items {
        let mut modd = Mod::default();
        modd.set_steam_id(Some(workshop_item.published_file_id.to_string()));

        modd.set_name(workshop_item.title.to_owned());
        modd.set_creator(workshop_item.owner.to_string());
        modd.set_file_name(workshop_item.file_name.to_owned());
        modd.set_file_size(workshop_item.file_size as u64);
        modd.set_description(workshop_item.description.to_owned());
        modd.set_time_created(workshop_item.time_created as usize);
        modd.set_time_updated(workshop_item.time_updated as usize);

        mods.push(modd);
    }

    Ok(mods)
}

pub fn request_mods_data_raw(game: &GameInfo, mod_ids: &[String]) -> Result<Vec<QueryResultDerive>> {

    // Do not call the cmd if there are no mods.
    if mod_ids.is_empty() {
        return Ok(vec![])
    }

    let game_path = setting_path(game.key());
    let steam_id = game.steam_id(&game_path)? as u32;
    let published_file_ids = mod_ids.join(",");
    let ipc_channel = rand::random::<u64>().to_string();

    let command_string = format!("get-published-file-details -s {steam_id} -p {published_file_ids} -i {ipc_channel}");
    let mut command = build_command_from_str(&command_string, BAT_GET_PUBLISHED_FILE_DETAILS, false, false)?;
    command.spawn()?;

    let channel = ipc_channel.to_ns_name::<GenericNamespaced>()?;
    let server = ListenerOptions::new().name(channel).create_sync()?;
    let mut stream = server.accept()?;

    let mut message = String::new();
    stream.read_to_string(&mut message)?;

    if message == "{}" {
        Err(anyhow!("Error retrieving Steam Workshop data."))
    } else {
        serde_json::from_str(&message).map_err(From::from)
    }
}

pub fn request_user_names(user_ids: &[String]) -> Result<HashMap<String, String>> {

    // Do not call the cmd if there are no users.
    if user_ids.is_empty() {
        return Ok(HashMap::new())
    }

    let mut client = Workshop::new(None);
    let api_key = setting_string("steam_api_key");
    if !api_key.is_empty() {
        client.set_apikey(Some(api_key));
        get_player_names(&client, user_ids)
    } else {
        Ok(HashMap::new())
    }
}

pub fn populate_mods_with_online_data(mods: &mut HashMap<String, Mod>, workshop_items: &[Mod]) -> Result<()> {
    for workshop_item in workshop_items {
        if let Some(modd) = mods.values_mut()
            .filter(|modd| modd.steam_id().is_some())
            .find(|modd| modd.steam_id() == workshop_item.steam_id()) {

            modd.set_name(workshop_item.name().to_string());
            modd.set_creator(workshop_item.creator().to_string());
            modd.set_file_name(workshop_item.file_name().to_string());
            modd.set_file_size(*workshop_item.file_size());
            modd.set_description(workshop_item.description().to_string());
            modd.set_time_created(*workshop_item.time_created());
            modd.set_time_updated(*workshop_item.time_updated());
        }
    }

    let user_ids = mods.values()
        .filter_map(|modd| if !modd.creator().is_empty() {
            Some(modd.creator().to_owned())
        } else { None }
        ).collect::<Vec<_>>();

    if user_ids.is_empty() {
        return Ok(());
    }

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
pub fn upload_mod_to_workshop(game: &GameInfo, modd: &Mod, title: &str, description: &str, tags: &[String], changelog: &str, visibility: &Option<u32>, force_update: bool) -> Result<()> {
    let game_path = setting_path(game.key());
    let steam_id = game.steam_id(&game_path)? as u32;

    let pack_path = if modd.paths().is_empty() {
        return Err(anyhow!("Mod Path not found."));
    } else {
        path_to_absolute_string(&modd.paths()[0])
    };

    // If we're force-updating (the default) we just open and resave the pack to update the timestamp so steam detects it as different.
    if force_update {
        let extra_data = Some(EncodeableExtraData::new_from_game_info(game));
        let mut pack = Pack::read_and_merge(&[PathBuf::from(&pack_path)], game, true, false, false)?;
        pack.save(None, game, &extra_data)?;
    }

    // If we have a published_file_id, it means this file exists in the workshop.
    //
    // So, instead of uploading, we just update it.
    let mut command_string = format!("{} -b -s {steam_id} -f \"{pack_path}\" -t {} --tags \"{}\"",
        match modd.steam_id() {
            Some(published_file_id) => format!("update --published-file-id {published_file_id}"),
            None => "upload".to_string(),
        },
        BASE64_STANDARD.encode(title),
        tags.join(",")
    );

    if !description.is_empty() {
        command_string.push_str(&format!(" -d {}", BASE64_STANDARD.encode(description)));
    }

    if !changelog.is_empty() {
        command_string.push_str(&format!(" -c {}", BASE64_STANDARD.encode(changelog)));
    }

    if let Some(visibility) = visibility {
        command_string.push_str(&format!(" --visibility {visibility}"));
    }

    let mut command = build_command_from_str(&command_string, BAT_UPLOAD_TO_WORKSHOP, false, true)?;
    command.spawn()?;

    Ok(())
}

/// This function launches a game through workshopper, with access to the Steam Api.
pub fn launch_game(game: &GameInfo, command_to_pass: &str, wait_for_finish: bool) -> Result<()> {
    let game_path = setting_path(game.key());
    let steam_id = game.steam_id(&game_path)? as u32;

    let command_string = format!("launch -b -s {steam_id} -c {command_to_pass}");
    let mut command = build_command_from_str(&command_string, BAT_LAUNCH_GAME, false, false)?;
    let mut handle = command.spawn()?;

    if wait_for_finish {
        let _ = handle.wait();
    }

    Ok(())
}

/// This function asks workshopper to get all subscribed items, check which ones are missing, and tell steam to re-download them.
pub fn download_subscribed_mods(game: &GameInfo, published_file_ids: &Option<Vec<String>>) -> Result<()> {
    let game_path = setting_path(game.key());
    let steam_id = game.steam_id(&game_path)? as u32;

    let mut command_string = format!("download-subscribed-items -s {steam_id}");
    if let Some(published_file_ids) = published_file_ids {
        command_string.push_str(" -p ");
        command_string.push_str(&published_file_ids.join(","));
    }

    let mut command = build_command_from_str(&command_string, BAT_DOWNLOAD_SUBSCRIBED_ITEMS, true, false)?;
    let mut handle = command.spawn()?;
    handle.wait()?;

    Ok(())
}

pub fn user_id(game: &GameInfo) -> Result<u64> {
    let game_path = setting_path(game.key());
    let steam_id = game.steam_id(&game_path)? as u32;
    let ipc_channel = rand::random::<u64>().to_string();

    let command_string = format!("user-id -s {steam_id} -i {ipc_channel}");
    let mut command = build_command_from_str(&command_string, BAT_USER_ID, false, false)?;
    let _ = command.spawn()?;

    let channel = ipc_channel.to_ns_name::<GenericNamespaced>()?;
    let server = ListenerOptions::new().name(channel).create_sync()?;
    let mut stream = server.accept()?;

    let mut bytes = vec![];
    stream.read_to_end(&mut bytes)?;

    let array: [u8; 8] = bytes.try_into().map_err(|_| anyhow!("Error when trying to get the Steam User ID."))?;

    Ok(u64::from_le_bytes(array))
}

fn app_manifest_path(game: &GameInfo, game_path: &Path) -> Result<PathBuf> {
    let steam_id = game.steam_id(game_path)? as u32;
    let mut app_path = game_path.to_path_buf();
    app_path.pop();
    app_path.pop();

    app_path.push(format!("appmanifest_{}.acf", steam_id));
    Ok(app_path)
}

pub fn can_game_locked(game: &GameInfo, game_path: &Path) -> Result<bool> {
    let app_path = app_manifest_path(game, game_path)?;
    Ok(app_path.is_file())
}

pub fn is_game_locked(game: &GameInfo, game_path: &Path) -> Result<bool> {
    let app_path = app_manifest_path(game, game_path)?;
    if !app_path.is_file() {
        return Ok(false);
    }

    let metadata = app_path.metadata()?;
    let permissions = metadata.permissions();

    Ok(permissions.readonly())
}

pub fn toggle_game_locked(game: &GameInfo, game_path: &Path, toggle: bool) -> Result<bool> {
    let app_path = app_manifest_path(game, game_path)?;
    if !app_path.is_file() {
        return Ok(false);
    }

    let metadata = app_path.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_readonly(toggle);

    std::fs::set_permissions(app_path, permissions.clone())?;

    Ok(permissions.readonly())
}

fn build_command_from_str(cmd: &str, bat_name: &str, force_detached_process: bool, force_new_console: bool) -> Result<Command> {
    let cmd = format!("\"{}\" {cmd} & exit", WORKSHOPPER_PATH.as_str());

    let mut file = BufWriter::new(File::create(bat_name)?);
    file.write_all(cmd.as_bytes())?;
    file.flush()?;

    let mut command = Command::new("cmd");
    command.arg("/C");
    command.arg(bat_name);

    // This is for creating the terminal window under windows. Without it, the entire process
    // runs in the background and there's no feedback on when it's done.
    #[cfg(target_os = "windows")]if force_detached_process {
        command.creation_flags(DETACHED_PROCESS);
    } else if force_new_console {
        command.creation_flags(CREATE_NEW_CONSOLE);
    } else if cfg!(debug_assertions) {
        command.creation_flags(DETACHED_PROCESS);
    } else {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    Ok(command)
}
