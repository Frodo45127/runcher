//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a small CLI tool to interact with the Steam Workshop.
//!
//! While initially designed for Total War games... may work with any other game.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::type_complexity,                // Disabled due to useless warnings.
    clippy::too_many_arguments              // Disabled because it gets annoying really quick.
)]

use anyhow::Result;
use clap::Parser;
use steamworks::PublishedFileId;

use std::path::PathBuf;
use std::process::exit;

use rpfm_lib::integrations::log::*;

use crate::app::{Cli, Commands};

mod app;
mod commands;

/// Guess you know what this function does....
fn main() {

    let logger = Logger::init(&PathBuf::from("."), true, true, release_name!());
    if logger.is_err() {
        warn!("Logging initialization has failed. No logs will be saved.");
    }

    // Parse the entire cli command.
    let cli = Cli::parse();
    info!("{:?}", cli.command);

    // Execute the commands.
    let result: Result<()> = match cli.command {
        Commands::Upload { steam_id, pack_path, title, description, tags, changelog } => crate::commands::ugc::upload(steam_id, &pack_path, &title, &description, &tags, &changelog),
        Commands::Update { steam_id, published_file_id, pack_path, title, description, tags, changelog } => crate::commands::ugc::update(None, None, PublishedFileId(published_file_id), steam_id, &pack_path, &title, &description, &tags, &changelog)
    };

    // Output the result of the commands, then give people 30 seconds to read them before exiting.
    match result {
        Ok(_) => {
            info!("Done.");
            std::thread::sleep(std::time::Duration::from_millis(30000));
            exit(0)
        },
        Err(error) => {
            error!("{}", error.to_string());
            std::thread::sleep(std::time::Duration::from_millis(30000));
            exit(1);
        },
    }
}
