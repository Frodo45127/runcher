//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crossbeam::channel::Sender;

use std::path::PathBuf;

use rpfm_lib::integrations::log::*;
use rpfm_ui_common::settings::error_path;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::updater::check_updates_main_program;

/// This is the network loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub fn network_loop() {

    // Initalize background sentry guard. This should, in theory, register crashes on the background thread.
    let _sentry_guard = Logger::init(&error_path().unwrap_or_else(|_| PathBuf::from(".")), true, false);

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
            Command::CheckUpdates => {
                match check_updates_main_program() {
                    Ok(response) => CentralCommand::send_back(&sender, Response::APIResponse(response)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // If you hit this, you fucked it up somewhere else.
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }
    }
}
