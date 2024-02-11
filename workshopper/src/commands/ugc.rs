//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::{anyhow, Result};
use crossbeam::channel::{Sender, Receiver, TryRecvError, unbounded};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use steamworks::{AppId, Client, ClientManager, FileType, PublishedFileId, PublishedFileVisibility, SingleClient, UGC, UpdateStatus, UpdateWatchHandle};

use std::fmt::Write;
use std::path::Path;
use std::thread::JoinHandle;

use rpfm_lib::integrations::log::{error, info};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug)]
pub enum SteamWorksThreadMessage {
    PublishedFileId(PublishedFileId),
    Ok,
    Error(anyhow::Error),
    Exit
}

//---------------------------------------------------------------------------//
//                      UGC (Workshop) public functions
//---------------------------------------------------------------------------//

/// This function is used to upload a new mod to the Workshop. For updating mods, do not use this. Use update instead.
pub fn upload(
    steam_id: u32,
    pack_path: &Path,
    title: &str,
    description: &Option<String>,
    tags: &[String],
    changelog: &Option<String>
) -> Result<()> {

    // Initialize the API.
    let (client, tx, callback_thread) = init(steam_id)?;
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

    update(Some(Ok((client, tx, callback_thread))), Some(ugc), published_file_id, steam_id, pack_path, title, description, tags, changelog)
}

/// This function is used to update an existing mod on the Workshop. For new mods, do not use this. Use upload instead.
///
/// The first two arguments are for internal re-use of this function. Pass them as none if you're just calling this function to update a mod.
pub fn update(
    api: Option<Result<(Client, Sender<SteamWorksThreadMessage>, JoinHandle<()>)>>,
    ugc: Option<UGC<ClientManager>>,
    published_file_id: PublishedFileId,
    steam_id: u32,
    pack_path: &Path,
    title: &str,
    description: &Option<String>,
    tags: &[String],
    changelog: &Option<String>
) -> Result<()> {

    // Initialize the API.
    let (client, tx, callback_thread) = api.unwrap_or_else(|| init(steam_id))?;
    let ugc = ugc.unwrap_or_else(|| client.ugc());

    // Prepare the preview path. We replicate the same behavior as the vanilla launcher.
    let mut preview_pack = pack_path.to_path_buf();
    preview_pack.set_extension("png");

    let (tx_query, rx_query): (Sender<SteamWorksThreadMessage>, Receiver<SteamWorksThreadMessage>) = unbounded();
    let update_handle = upload_item_content(&ugc, tx_query, steam_id, published_file_id, pack_path, &preview_pack, title, description, tags, changelog, PublishedFileVisibility::Private);

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
                        bar.set_position(prev_total);
                    }

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
                                bar.set_position(prev_total);
                            }

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
                                bar.set_position(prev_total);
                            }

                            bar = None;

                            info!("Committing changes...");
                        }
                    },
                    UpdateStatus::Invalid => {
                        finish(tx, callback_thread)?;
                        return Err(anyhow!("Invalid UpdateStatus."));
                    },
                }

                std::thread::sleep(std::time::Duration::from_millis(20));
            },

            // This is a bug.
            Err(TryRecvError::Disconnected) => panic!("Thread disconected."),
        }
    }
}

//---------------------------------------------------------------------------//
//                      UGC (Workshop) private functions
//---------------------------------------------------------------------------//

/// This function initializes the client and callback thread. DO NOT CALL IT IF THERE'S ALREADY A CLIENT ALIVE.
fn init(steam_id: u32) -> Result<(Client, Sender<SteamWorksThreadMessage>, JoinHandle<()>)> {
    let (client, single) = Client::init_app(steam_id)?;
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
    pack_path: &Path,
    preview_path: &Path,
    title: &str,
    description: &Option<String>,
    tags: &[String],
    changelog: &Option<String>,
    visibility: PublishedFileVisibility,
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
        .content_path(pack_path)
        .preview_path(preview_path)
        .title(title)
        .tags(tags.to_vec(), false)
        .visibility(visibility);

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
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    bar
}
