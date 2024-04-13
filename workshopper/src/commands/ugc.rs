//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::{anyhow, Result};
use base64::prelude::*;
use crossbeam::channel::{Sender, Receiver, TryRecvError, unbounded};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use interprocess::local_socket::LocalSocketStream;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use steamworks::{AppId, Client, ClientManager, DownloadItemResult, FileType, PublishedFileId, PublishedFileVisibility, QueryResult, SingleClient, SteamId, UpdateStatus, UpdateWatchHandle, UGC};

use std::fmt::Write as FmtWrite;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use rpfm_lib::{games::GameInfo, integrations::log::{error, info, warn}};

const IPC_NAME_GET_PUBLISHED_FILE_DETAILS: &str = "runcher_get_published_file_details";

const TOTAL_WAR_BASE_TAG: &str = "mod";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryResultDerive {
    pub published_file_id: PublishedFileId,
    pub creator_app_id: Option<AppId>,
    pub consumer_app_id: Option<AppId>,
    pub title: String,
    pub description: String,
    pub owner: SteamId,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum PublishedFileVisibilityDerive {
    Public,
    FriendsOnly,
    Private,
    Unlisted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Debug)]
pub enum SteamWorksThreadMessage {
    QueryResults(Vec<QueryResult>),
    PublishedFileId(PublishedFileId),
    Ok,
    Error(anyhow::Error),
    Exit
}

//---------------------------------------------------------------------------//
//                           From Implementations
//---------------------------------------------------------------------------//

impl From<&QueryResult> for QueryResultDerive {
    fn from(value: &QueryResult) -> Self {
        Self {
            published_file_id: value.published_file_id.clone(),
            creator_app_id: value.creator_app_id.clone(),
            consumer_app_id: value.consumer_app_id.clone(),
            title: value.title.clone(),
            description: value.description.clone(),
            owner: value.owner.clone(),
            time_created: value.time_created.clone(),
            time_updated: value.time_updated.clone(),
            time_added_to_user_list: value.time_added_to_user_list.clone(),
            visibility: PublishedFileVisibilityDerive::from(value.visibility),
            banned: value.banned.clone(),
            accepted_for_use: value.accepted_for_use.clone(),
            tags: value.tags.clone(),
            tags_truncated: value.tags_truncated.clone(),
            file_name: value.file_name.to_owned(),
            file_type: FileTypeDerive::from(value.file_type),
            file_size: value.file_size.clone(),
            url: value.url.clone(),
            num_upvotes: value.num_upvotes.clone(),
            num_downvotes: value.num_downvotes.clone(),
            score: value.score.clone(),
            num_children: value.num_children.clone()
        }
    }
}

impl From<PublishedFileVisibility> for PublishedFileVisibilityDerive {
    fn from(value: PublishedFileVisibility) -> Self {
        match value {
            PublishedFileVisibility::Public => Self::Public,
            PublishedFileVisibility::FriendsOnly => Self::FriendsOnly,
            PublishedFileVisibility::Private => Self::Private,
            PublishedFileVisibility::Unlisted => Self::Unlisted,
        }
    }
}

impl From<FileType> for FileTypeDerive {
    fn from(value: FileType) -> Self {
        match value {
            FileType::Community => Self::Community,
            FileType::Microtransaction => Self::Microtransaction,
            FileType::Collection => Self::Collection,
            FileType::Art => Self::Art,
            FileType::Video => Self::Video,
            FileType::Screenshot => Self::Screenshot,
            FileType::Game => Self::Game,
            FileType::Software => Self::Software,
            FileType::Concept => Self::Concept,
            FileType::WebGuide => Self::WebGuide,
            FileType::IntegratedGuide => Self::IntegratedGuide,
            FileType::Merch => Self::Merch,
            FileType::ControllerBinding => Self::ControllerBinding,
            FileType::SteamworksAccessInvite => Self::SteamworksAccessInvite,
            FileType::SteamVideo => Self::SteamVideo,
            FileType::GameManagedItem => Self::GameManagedItem,
        }
    }
}

//---------------------------------------------------------------------------//
//                      UGC (Workshop) public functions
//---------------------------------------------------------------------------//

pub fn published_file_details(steam_id: u32, published_file_ids: &str) -> Result<()> {
    let mut published_file_ids_enums = vec![];
    let published_file_ids_split = published_file_ids.split(",").collect::<Vec<_>>();
    for id in &published_file_ids_split {
        info!("{}", &id);

        published_file_ids_enums.push(PublishedFileId(id.parse::<u64>()?));
    }

    // Initialize the API.
    let (client, tx, callback_thread) = init(steam_id, Some(IPC_NAME_GET_PUBLISHED_FILE_DETAILS))?;
    let ugc = client.ugc();

    // Create the query and request the results.
    let (tx_query, rx_query): (Sender<SteamWorksThreadMessage>, Receiver<SteamWorksThreadMessage>) = unbounded();
    get_published_file_details(&ugc, tx_query, published_file_ids_enums);

    let response = rx_query.recv()?;
    match response {
        SteamWorksThreadMessage::QueryResults(results) => {
            let results = results.iter().map(|result| QueryResultDerive::from(result)).collect::<Vec<_>>();
            if let Ok(message) = to_string_pretty(&results) {

                if let Ok(mut stream) = LocalSocketStream::connect(IPC_NAME_GET_PUBLISHED_FILE_DETAILS) {
                    let _ = stream.write(message.as_bytes());
                }

                // In debug mode, dump the response to a file so we can see errors on it.
                if cfg!(debug_assertions) {
                    let path = PathBuf::from("get_published_file_details.json");
                    let mut file = BufWriter::new(File::create(path)?);
                    file.write_all(to_string_pretty(&results)?.as_bytes())?;
                    file.flush()?;
                }
            } else {
                if let Ok(mut stream) = LocalSocketStream::connect(IPC_NAME_GET_PUBLISHED_FILE_DETAILS) {
                    let _ = stream.write(b"{}");
                }
            }

            return finish(tx, callback_thread)
        },
        SteamWorksThreadMessage::Error(error) => {

            if let Ok(mut stream) = LocalSocketStream::connect(IPC_NAME_GET_PUBLISHED_FILE_DETAILS) {
                let _ = stream.write(b"{}");
            }

            finish(tx, callback_thread)?;
            return Err(error)
        },
        _ => panic!("{response:?}")
    };
}

/// This function is used to upload a new mod to the Workshop. For updating mods, do not use this. Use update instead.
pub fn upload(
    base64: bool,
    steam_id: u32,
    pack_path: &Path,
    title: &str,
    description: &Option<String>,
    tags: &[String],
    changelog: &Option<String>,
    visibility: &Option<u32>,
) -> Result<()> {

    // Initialize the API.
    let (client, tx, callback_thread) = init(steam_id, None)?;
    let ugc = client.ugc();

    // Create the item.
    let (tx_query, rx_query): (Sender<SteamWorksThreadMessage>, Receiver<SteamWorksThreadMessage>) = unbounded();
    create_item(&ugc, tx_query, steam_id);

    let response = rx_query.recv()?;
    let published_file_id = match response {
        SteamWorksThreadMessage::PublishedFileId(id) => id,
        SteamWorksThreadMessage::Error(error) => {
            finish(tx, callback_thread)?;
            return Err(error)
        },
        _ => panic!("{response:?}")
    };

    // We need to subscribe ourself to the item. Otherwise we'll not get it's data in a data request.
    let (tx_query, rx_query): (Sender<SteamWorksThreadMessage>, Receiver<SteamWorksThreadMessage>) = unbounded();
    subscribe_item(&ugc, tx_query, published_file_id);

    let response = rx_query.recv()?;
    match response {
        SteamWorksThreadMessage::Ok => {},
        SteamWorksThreadMessage::Error(error) => {
            finish(tx, callback_thread)?;
            return Err(error)
        },
        _ => panic!("{response:?}")
    };

    // Finally update it with the local file.
    update(Some(Ok((client, tx, callback_thread))), Some(ugc), base64, published_file_id, steam_id, pack_path, title, description, tags, changelog, visibility)
}

/// This function is used to update an existing mod on the Workshop. For new mods, do not use this. Use upload instead.
///
/// The first two arguments are for internal re-use of this function. Pass them as none if you're just calling this function to update a mod.
pub fn update(
    api: Option<Result<(Client, Sender<SteamWorksThreadMessage>, JoinHandle<()>)>>,
    ugc: Option<UGC<ClientManager>>,
    base64: bool,
    published_file_id: PublishedFileId,
    steam_id: u32,
    pack_path: &Path,
    title: &str,
    description: &Option<String>,
    tags: &[String],
    changelog: &Option<String>,
    visibility: &Option<u32>,
) -> Result<()> {

    // Initialize the API.
    let (client, tx, callback_thread) = api.unwrap_or_else(|| init(steam_id, None))?;
    let ugc = ugc.unwrap_or_else(|| client.ugc());

    // Sanitize the pack_path.
    let pack_path = if pack_path.to_string_lossy().starts_with("\\\\?\\") {
        PathBuf::from(pack_path.to_string_lossy()[4..].to_owned())
    } else {
        pack_path.to_path_buf()
    };

    // Prepare the preview path. We replicate the same behavior as the vanilla launcher.
    let mut preview_path = pack_path.to_path_buf();
    preview_path.set_extension("png");

    let (tx_query, rx_query): (Sender<SteamWorksThreadMessage>, Receiver<SteamWorksThreadMessage>) = unbounded();

    // If we're in base64 mode, decode the problematic fields.
    let title = if base64 {
        String::from_utf8(BASE64_STANDARD.decode(title)?)?
    } else {
        title.to_owned()
    };

    let mut description = description.clone();
    let mut changelog = changelog.clone();
    if base64 {
        if let Some(ref mut description) = description {
            *description = String::from_utf8(BASE64_STANDARD.decode(description.clone())?)?;
        }

        if let Some(ref mut changelog) = changelog {
            *changelog = String::from_utf8(BASE64_STANDARD.decode(changelog.clone())?)?;
        }
    }

    // TODO: Make this only trigger when doing it on a Total War game.

    // NOTE: CA seems to be doing a "copy pack and preview to folder, then upload" thing, to get both uploaded.
    // We want to keep this behavior because otherwise downloaded mods have no preview.
    let upload_path = if cfg!(debug_assertions) {
        PathBuf::from("./mod_uploads/")
    } else {
        let mut upload_path = std::env::current_exe().unwrap();
        upload_path.pop();
        upload_path.push("mod_uploads");
        upload_path
    };

    info!("Copying pack and preview from {} to {}", pack_path.to_string_lossy(), upload_path.to_string_lossy());

    // Clean the mod_uploads folder.
    if upload_path.is_dir() {
        std::fs::remove_dir_all(&upload_path)?;
    }

    DirBuilder::new().recursive(true).create(&upload_path)?;

    // Copy the pack and preview to the upload folder.
    let mut pack_path_dest = upload_path.to_path_buf();
    pack_path_dest.push(pack_path.file_name().unwrap());

    let mut preview_path_dest = upload_path.to_path_buf();
    preview_path_dest.push(preview_path.file_name().unwrap());

    std::fs::copy(&pack_path, pack_path_dest)?;
    std::fs::copy(&preview_path, &preview_path_dest)?;

    info!("Copying done, preparing upload.");

    let update_handle = upload_item_content(&ugc, tx_query, steam_id, published_file_id, &upload_path, &preview_path, &title, &description, tags, &changelog, visibility);

    // Initialize the progress bar. The upload is a 5-step process, and the bar should come at 3 and 4.
    let mut bar: Option<ProgressBar> = None;
    let mut prev_status = UpdateStatus::Invalid;
    let mut prev_total = 0;

    // We loop keeping painting the progress to the terminal until we're done.
    loop {

        match rx_query.try_recv() {
            Ok(response) => match response {
                SteamWorksThreadMessage::Ok => {

                    // If stuff happened too quickly and the commit didn't trigger, do it here.
                    if let Some(ref bar) = bar {
                        bar.finish();
                    }

                    info!("Upload done, deleting temp files.");

                    // Delete the folder once it's done so we don't occupy space we shouldn't.
                    if upload_path.is_dir() {
                        std::fs::remove_dir_all(&upload_path)?;
                    }

                    info!("Temp files deleted.");

                    return finish(tx, callback_thread)
                },
                SteamWorksThreadMessage::Error(error) => {
                    finish(tx, callback_thread)?;
                    return Err(error)
                },
                _ => panic!("{response:?}")
            }

            // If it's empty, paint to the console the progress, wait 20 ms and try again.
            Err(TryRecvError::Empty) => {
                let (status, loaded, total) = update_handle.progress();
                match status {
                    UpdateStatus::PreparingConfig => {
                        if prev_status != UpdateStatus::PreparingConfig {
                            prev_status = UpdateStatus::PreparingConfig;
                            info!("Preparing config...");
                        }
                    },
                    UpdateStatus::PreparingContent => {
                        if prev_status != UpdateStatus::PreparingContent {
                            prev_status = UpdateStatus::PreparingContent;
                            info!("Preparing content...");
                        }
                    },
                    UpdateStatus::UploadingContent => {
                        if prev_status != UpdateStatus::UploadingContent {
                            prev_status = UpdateStatus::UploadingContent;
                        }

                        // Total takes some time to update after changing status.
                        if prev_total == 0 && total > 0 {
                            info!("Uploading content of size: {}.", total);
                            prev_total = total;
                            bar = Some(progress_bar(total));
                        }

                        if let Some(ref bar) = bar {
                            bar.set_position(loaded);
                        }
                    },
                    UpdateStatus::UploadingPreviewFile => {
                        if prev_status != UpdateStatus::UploadingPreviewFile {
                            prev_status = UpdateStatus::UploadingPreviewFile;

                            // Fill the previous bar before making the new one.
                            if let Some(ref bar) = bar {
                                bar.finish();
                            }

                            prev_total = 0;
                        }

                        // Total takes some time to update after changing status.
                        if prev_total == 0 && total > 0 {
                            info!("Uploading preview file of size: {}.", total);
                            prev_total = total;
                            bar = Some(progress_bar(total));
                        }

                        if let Some(ref bar) = bar {
                            bar.set_position(loaded);
                        }
                    },
                    UpdateStatus::CommittingChanges => {
                        if prev_status != UpdateStatus::CommittingChanges {
                            prev_status = UpdateStatus::CommittingChanges;

                            // Fill the previous bar before killing it.
                            if let Some(ref bar) = bar {
                                bar.finish();
                            }

                            bar = None;

                            info!("Committing changes...");
                        }
                    },

                    // Invalid is usually completed. So just return Ok.
                    UpdateStatus::Invalid => {
                        info!("Invalid UpdateStatus. This is an error, or the upload finished.");
                    },
                }

                std::thread::sleep(std::time::Duration::from_millis(20));
            },

            // This is a bug.
            Err(TryRecvError::Disconnected) => panic!("Thread disconected."),
        }
    }
}

/// This function tries to download all mods a user has subscribed to from a game.
pub fn download_subscribed_mods(steam_id: u32, published_file_ids: Option<String>) -> Result<()> {

    // Initialize the API.
    let (client, tx, callback_thread) = init(steam_id, None)?;
    let ugc = client.ugc();

    // Get the published_file_ids.
    let published_file_ids = match published_file_ids {
        Some(ids) => ids.split(",").filter_map(|x| x.parse::<u64>().ok()).map(|x| PublishedFileId(x)).collect(),
        None => ugc.subscribed_items(),
    };

    for published_file_id in published_file_ids {

        if ugc.download_item(published_file_id, true) {
            info!("Downloading workshop item with ID: {}.", published_file_id.0);

            let (tx_callback, rx_callback): (Sender<SteamWorksThreadMessage>, Receiver<SteamWorksThreadMessage>) = unbounded();
            let _cb = client.register_callback(move |d: DownloadItemResult| {
                match d.error {
                    Some(error) => {
                        error!("Error downloading workshop item with ID {}: {}", published_file_id.0, error);
                        let _ = tx_callback.send(SteamWorksThreadMessage::Error(error.into()));
                    }
                    None => {
                        info!("Workshop item with ID {} downloaded.", published_file_id.0);
                        let _ = tx_callback.send(SteamWorksThreadMessage::Ok);
                    }
                }
            });

            let response = rx_callback.recv()?;
            match response {
                SteamWorksThreadMessage::Ok => {
                    if let Some(install_info) = ugc.item_install_info(published_file_id) {

                        // So, fun bug: if the item is a legacy item and somehow it got deleted from the content folder, steam will consistently fail to re-download it.
                        // Solution? Unsubscribe, then resubscribe, then download again. Fuck legacy mods.
                        if install_info.folder.ends_with(".bin") && !PathBuf::from(&install_info.folder).is_file() {
                            warn!("Steam lied about downloading Workshop item with ID {}. Posible legacy mod.", published_file_id.0);
                            warn!("To re-download this one, go to https://steamcommunity.com/sharedfiles/filedetails/?id={}, then unsubscribe and re-subscribe.", published_file_id.0);
                        }
                    }
                    continue
                },
                SteamWorksThreadMessage::Error(_) => continue,
                _ => panic!("{response:?}")
            };
        }
    }

    finish(tx, callback_thread)?;
    Ok(())
}

//---------------------------------------------------------------------------//
//                      UGC (Workshop) private functions
//---------------------------------------------------------------------------//

/// This function initializes the client and callback thread. DO NOT CALL IT IF THERE'S ALREADY A CLIENT ALIVE.
fn init(steam_id: u32, channel: Option<&str>) -> Result<(Client, Sender<SteamWorksThreadMessage>, JoinHandle<()>)> {
    let (client, single) = match Client::init_app(steam_id) {
        Ok(client) => client,
        Err(error) => {
            if let Some(channel) = channel {
                if let Ok(mut stream) = LocalSocketStream::connect(channel) {
                    let _ = stream.write("{}".as_bytes());
                }
            }

            return Err(From::from(error));
        }
    };
    let (tx, rx) = unbounded();

    let thread = std::thread::spawn(move || { callback_loop(single, rx); });

    Ok((client, tx, thread))
}

fn callback_loop(single: SingleClient<ClientManager>, rx: Receiver<SteamWorksThreadMessage>) {

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//
    info!("SteamWorks Callback looping around…");
    loop {

        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // check if the channel is closed or if there is a message
        // end the thread if either is true
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {}
        }
    }
}

/// Use this to close the callback thread.
fn finish(tx: Sender<SteamWorksThreadMessage>, callback_thread: JoinHandle<()>) -> Result<()> {
    tx.send(SteamWorksThreadMessage::Exit)?;
    callback_thread.join().unwrap();
    Ok(())
}

/// Function to subscribe to an specific item in the workshop.
///
/// This function does NOT finish the background thread.
fn subscribe_item(ugc: &UGC<ClientManager>, sender: Sender<SteamWorksThreadMessage>, published_file_id: PublishedFileId) {
    ugc.subscribe_item(
        published_file_id,
        move |result| {
            match result {
                Ok(_) => {
                    info!("Subscribed Workshop item with ID {}.", published_file_id.0);
                    let _ = sender.send(SteamWorksThreadMessage::Ok);
                }

                Err(error) => {
                    error!("Failed to subscribe to Workshop item with ID {}: {}.", published_file_id.0, error.to_string());
                    let _ = sender.send(SteamWorksThreadMessage::Error(From::from(error)));
                },
            }
        },
    );
}

/// Function to unsubscribe from an specific item in the workshop.
///
/// This function does NOT finish the background thread.
fn unsubscribe_item(ugc: &UGC<ClientManager>, sender: Sender<SteamWorksThreadMessage>, published_file_id: PublishedFileId) {
    ugc.unsubscribe_item(
        published_file_id,
        move |result| {
            match result {
                Ok(_) => {
                    info!("Unsubscribed Workshop item with ID {}.", published_file_id.0);
                    let _ = sender.send(SteamWorksThreadMessage::Ok);
                }

                Err(error) => {
                    info!("Failed to unsubscribe to Workshop item with ID {}: {}.", published_file_id.0, error.to_string());
                    let _ = sender.send(SteamWorksThreadMessage::Error(From::from(error)));
                },
            }
        },
    );
}

/// Function to retrieve the detailed data corresponding to a list of PublishedFileIds.
fn get_published_file_details(ugc: &UGC<ClientManager>, sender: Sender<SteamWorksThreadMessage>, published_file_ids: Vec<PublishedFileId>) {
    match ugc.query_items(published_file_ids) {
        Ok(handle) => {
            handle.include_long_desc(true)
                .fetch(move |results| {
                    match results {
                        Ok(results) => {
                            info!("Mod list data retireved from workshop.");

                            // We need to process the results before sending them.
                            let mut processed_results = vec![];
                            for result in results.iter() {
                                if let Some(result) = result {
                                    processed_results.push(result);
                                }
                            }

                            let _ = sender.send(SteamWorksThreadMessage::QueryResults(processed_results));
                        }

                        Err(error) => {
                            error!("get-published-file-details call failed: {}", error);
                            let _ = sender.send(SteamWorksThreadMessage::Error(From::from(error)));
                        },
                    }
                },);
            }
        Err(error) => { let _ = sender.send(SteamWorksThreadMessage::Error(From::from(error))); },
    }
}

/// Function to create an item in a specific workshop.
///
/// This only creates the item. You need to upload a pack after this.
fn create_item(ugc: &UGC<ClientManager>, sender: Sender<SteamWorksThreadMessage>, app_id: u32) {
    ugc.create_item(
        AppId(app_id),
        FileType::Community,
        move |create_result| {

            match create_result {
                Ok((published_id, needs_to_agree_to_terms)) => {

                    if needs_to_agree_to_terms {
                        info!("You need to agree to the terms of use before you can upload any files");
                    }

                    info!("Published item with id {:?}", published_id);
                    let _ = sender.send(SteamWorksThreadMessage::PublishedFileId(published_id));
                }

                Err(error) => { let _ = sender.send(SteamWorksThreadMessage::Error(From::from(error))); },
            }
        },
    );
}

/// Function to upload an item to the workshop. This requires the item to already exists on the workshop.
fn upload_item_content(
    ugc: &UGC<ClientManager>,
    sender: Sender<SteamWorksThreadMessage>,
    app_id: u32,
    published_id: PublishedFileId,
    upload_path: &Path,
    preview_path: &Path,
    title: &str,
    description: &Option<String>,
    tags: &[String],
    changelog: &Option<String>,
    visibility: &Option<u32>,
) -> UpdateWatchHandle<ClientManager> {

    // uploading the content of the workshop item
    // this process uses a builder pattern to set properties of the item
    // mandatory properties are:
    // - title
    // - description
    // - preview_path
    // - content_path
    // - visibility
    // after setting the properties, call .submit() to start uploading the item
    // this function is unique in that it returns a handle to the upload, which can be used to
    // monitor the progress of the upload and needs a closure to be called when the upload is done
    // in this example, the watch handle is ignored for simplicity
    //
    // notes:
    // - once an upload is started, it cannot be cancelled!
    // - content_path is the path to a folder which houses the content you wish to upload
    let mut handle = ugc.start_item_update(AppId(app_id), published_id)
        .content_path(upload_path)
        .preview_path(preview_path)
        .title(title);

    if let Some(visibility) = visibility {
        handle = handle.visibility(match visibility {
            0 => PublishedFileVisibility::Public,
            1 => PublishedFileVisibility::FriendsOnly,
            2 => PublishedFileVisibility::Private,
            3 => PublishedFileVisibility::Unlisted,
            _ => panic!("Invalid Visibility"),
        });
    }

    let mut tags = tags.to_vec();

    // This is a Total War-specific limit. For other games, ignore the tag sanitizing.
    if let Ok(game) = GameInfo::game_by_steam_id(app_id as u64) {
        if let Ok(valid_tags) = game.steam_workshop_tags() {

            // NOTE: Tags are tricky. All mods uploaded to the workshop contain two tags: "mod" and one from a list of available tags.
            // And CA don't want people adding custom tags to the workshop. So we need to limit it to one tag from the list of existing tags.
            tags.retain(|tag| valid_tags.contains(tag));

            // Remove duplicated tags.
            tags.sort();
            tags.dedup();

            // "mod" has to be the first tag. The second one is user-chosen.
            if let Some(pos) = tags.iter().position(|x| x == TOTAL_WAR_BASE_TAG) {
                tags.remove(pos);
            }

            tags.insert(0, TOTAL_WAR_BASE_TAG.to_owned());

            // Reduce if we have more than two tags, trim it.
            if tags.len() > 2 {
                let _ = tags.split_off(2);
            }

            // If all tags got deleted and we only have mod, add the first one from the list of valid tags.
            if tags.len() == 1 {
                tags.push(valid_tags.first().unwrap().to_owned());
            }
        }
    }

    handle = handle.tags(tags, false);

    // TODO: Check if description is really mandatory.
    if let Some(ref description) = description {
        handle = handle.description(description);
    }

    handle.submit(changelog.as_deref(),
        move |upload_result| {
            match upload_result {
                Ok((published_id, needs_to_agree_to_terms)) => {

                    // If this is true here, it's an error.
                    if needs_to_agree_to_terms {
                        let error = "You need to agree to the terms of use before you can upload any files.";
                        error!("{}", error);
                        let _ = sender.send(SteamWorksThreadMessage::Error(anyhow!(error)));
                    }

                    // If not, the file was uploaded successfully.
                    else {
                        info!("Uploaded item with id {:?}", published_id);
                        let _ = sender.send(SteamWorksThreadMessage::Ok);
                    }
                }
                Err(error) => { let _ = sender.send(SteamWorksThreadMessage::Error(From::from(error))); },
            }
        }
    )
}

/// This just initializes a nice progress bar for the uploads.
fn progress_bar(total: u64) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn FmtWrite| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    bar
}
