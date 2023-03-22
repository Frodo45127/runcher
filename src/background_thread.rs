//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, &which can be &found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use base64::{Engine as _, engine::general_purpose};
use crossbeam::channel::Sender;
use rayon::prelude::*;
use zstd::stream::*;

use std::path::PathBuf;

use rpfm_lib::integrations::log::*;

use rpfm_ui_common::settings::error_path;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::integrations::*;

/// This is the background loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub fn background_loop() {

    // Initalize background sentry guard. This should, in theory, register crashes on the background thread.
    let _sentry_guard = Logger::init(&error_path().unwrap_or_else(|_| PathBuf::from(".")), true, false);

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
                match crate::updater::update_main_program() {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::GetStringFromLoadOrder(game_config) => {

                // Pre-sort the mods.
                let mut mods = game_config.mods()
                    .par_iter()
                    .map(|(_, modd)| modd)
                    .filter(|modd| *modd.enabled() && !modd.paths().is_empty())
                    .map(ShareableMod::from)
                    .collect::<Vec<_>>();

                mods.sort_by_key(|a| a.id().clone());

                let mods = serde_json::to_string(&mods).unwrap();
                let mut compressed = vec![];
                copy_encode(mods.as_bytes(), &mut compressed, 3).unwrap();

                let encoded: String = general_purpose::STANDARD_NO_PAD.encode(compressed);
                CentralCommand::send_back(&sender, Response::String(encoded));
            }
            Command::GetLoadOrderFromString(string) => {
                let debased = general_purpose::STANDARD_NO_PAD.decode(string.as_bytes()).unwrap();
                let mut decompressed = vec![];

                copy_decode(debased.as_slice(), &mut decompressed).unwrap();
                let mods: Vec<ShareableMod> = serde_json::from_slice(&decompressed).unwrap();

                CentralCommand::send_back(&sender, Response::VecShareableMods(mods));
            }

            Command::CheckUpdates => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }
    }
}
