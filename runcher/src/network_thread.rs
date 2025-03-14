//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crossbeam::channel::Sender;

use std::path::PathBuf;

use common_utils::updater::Updater;

use rpfm_lib::integrations::{git::*, log::*};
use rpfm_lib::schema::*;
use rpfm_ui_common::settings::error_path;

use crate::{sql_scripts_remote_path, CENTRAL_COMMAND, SQL_SCRIPTS_BRANCH, SQL_SCRIPTS_REMOTE, SQL_SCRIPTS_REPO};
use crate::communications::*;
use crate::mod_manager::integrations::request_mods_data;
use crate::settings_ui::schemas_path;
use crate::{REPO_NAME, REPO_OWNER};

/// This is the network loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub fn network_loop() {

    // Initalize background sentry guard. This should, in theory, register crashes on the background thread.
    let _sentry_guard = Logger::init(&error_path().unwrap_or_else(|_| PathBuf::from(".")), true, false, release_name!());

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//
    info!("Network Thread looping around…");
    loop {

        // Wait until you get something through the channel. This hangs the thread until we got something,
        // so it doesn't use processing power until we send it a message.
        let (sender, response): (Sender<Response>, Command) = CENTRAL_COMMAND.recv_network();
        match response {

            // Command to close the thread.
            Command::Exit => return,

            // When we want to check if there is an update available for RPFM...
            Command::CheckUpdates(channel) => {
                let updater = Updater::new(channel, REPO_OWNER, REPO_NAME);
                match updater.check(env!("CARGO_PKG_VERSION")) {
                    Ok(response) => CentralCommand::send_back(&sender, Response::APIResponse(response)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to check if there is a schema's update available...
            Command::CheckSchemaUpdates => {
                match schemas_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE);
                        match git_integration.check_update() {
                            Ok(response) => CentralCommand::send_back(&sender, Response::APIResponseGit(response)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to check if there is a schema's update available...
            Command::CheckSqlScriptsUpdates => {
                match sql_scripts_remote_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, SQL_SCRIPTS_REPO, SQL_SCRIPTS_BRANCH, SQL_SCRIPTS_REMOTE);
                        match git_integration.check_update() {
                            Ok(response) => CentralCommand::send_back(&sender, Response::APIResponseGit(response)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::RequestModsData(game, mod_ids) => {
                let request = request_mods_data(&game, &mod_ids);
                match request {
                    Ok(mods_data) => CentralCommand::send_back(&sender, Response::VecMod(mods_data)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // If you hit this, you fucked it up somewhere else.
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }
    }
}
