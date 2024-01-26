//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, &which can be &found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use crossbeam::channel::Sender;
use rayon::prelude::*;
use zstd::stream::*;

use std::path::PathBuf;

use rpfm_lib::integrations::{git::*, log::*};
use rpfm_lib::schema::*;

use rpfm_ui_common::settings::error_path;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::games::{TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE};
use crate::mod_manager::{game_config::GameConfig, mods::ShareableMod};
use crate::settings_ui::{schemas_path, translations_remote_path};
use crate::SCHEMA;

/// This is the background loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub fn background_loop() {

    // Initalize background sentry guard. This should, in theory, register crashes on the background thread.
    let _sentry_guard = Logger::init(&error_path().unwrap_or_else(|_| PathBuf::from(".")), true, false, release_name!());

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//
    info!("Background Thread looping around…");
    loop {

        // Wait until you get something through the channel. This hangs the thread until we got something,
        // so it doesn't use processing power until we send it a message.
        let (sender, response): (Sender<Response>, Command) = CENTRAL_COMMAND.recv_background();
        match response {

            Command::Exit => return,

            Command::UpdateMainProgram => {
                match crate::updater_ui::update_main_program() {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::UpdateSchemas(schema_file_name) => {
                match schemas_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE);
                        match git_integration.update_repo() {
                            Ok(_) => {
                                let schema_path = schemas_path().unwrap().join(schema_file_name);
                                *SCHEMA.write().unwrap() = Schema::load(&schema_path, None).ok();
                                CentralCommand::send_back(&sender, Response::Success)
                            },
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::UpdateTranslations => {
                match translations_remote_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE);
                        match git_integration.update_repo() {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::GetStringFromLoadOrder(game_config) => {
                match get_string_from_load_order(game_config) {
                    Ok(encoded) => CentralCommand::send_back(&sender, Response::String(encoded)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::GetLoadOrderFromString(string) => {
                match get_load_order_from_string(string) {
                    Ok(mods) => CentralCommand::send_back(&sender, Response::VecShareableMods(mods)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::CheckUpdates | Command::CheckSchemaUpdates | Command::CheckTranslationsUpdates | Command::RequestModsData(_) => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }
    }
}

fn get_string_from_load_order(game_config: GameConfig) -> Result<String> {
    let mut mods = game_config.mods()
        .par_iter()
        .map(|(_, modd)| modd)
        .filter(|modd| *modd.enabled() && !modd.paths().is_empty())
        .map(ShareableMod::from)
        .collect::<Vec<_>>();

    mods.sort_by_key(|a| a.id().clone());

    let mods = serde_json::to_string(&mods)?;
    let mut compressed = vec![];
    copy_encode(mods.as_bytes(), &mut compressed, 3)?;

    Ok(general_purpose::STANDARD_NO_PAD.encode(compressed))
}

fn get_load_order_from_string(string: String) -> Result<Vec<ShareableMod>> {
    let debased = general_purpose::STANDARD_NO_PAD.decode(string.as_bytes())?;
    let mut decompressed = vec![];

    copy_decode(debased.as_slice(), &mut decompressed)?;
    serde_json::from_slice(&decompressed).map_err(From::from)
}
