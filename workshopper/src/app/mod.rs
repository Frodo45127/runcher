//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the input and command definitions for the tool.

use clap::{Parser, Subcommand};

use std::path::PathBuf;

//---------------------------------------------------------------------------//
//                          Struct/Enum Definitions
//---------------------------------------------------------------------------//

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {

    GetPublishedFileDetails {

        /// SteamId/AppId of the game we're going to upload the mod for.
        #[arg(short, long, value_name = "STEAM_ID")]
        steam_id: u32,

        /// List of published file ids, separated by comma.
        #[arg(short, long, required = true, value_name = "PUBLISHED_FILE_IDS")]
        published_file_ids: String,
    },

    Upload {

        /// SteamId/AppId of the game we're going to upload the mod for.
        #[arg(short, long, value_name = "STEAM_ID")]
        steam_id: u32,

        /// Path of the file (Pack in Total War) this operation will use.
        #[arg(short, long, required = true, value_name = "PATH")]
        file_path: PathBuf,

        /// Title the mod will receive.
        #[arg(short, long, required = true, value_name = "TITLE")]
        title: String,

        /// Description the mod will receive.
        #[arg(short, long, required = false, value_name = "DESCRIPTION")]
        description: Option<String>,

        /// Tags the mod will receive.
        #[arg(long, required = true, value_name = "COMMA-SEPARATED TAGS")]
        tags: Vec<String>,

        /// Changelog for the initial release.
        #[arg(short, long, required = false, value_name = "CHANGELOG")]
        changelog: Option<String>
    },

    Update {

        /// SteamId/AppId of the game we're going to upload the mod for.
        #[arg(short, long, value_name = "STEAM_ID")]
        steam_id: u32,

        /// PublishedFileId of the mod we're updating.
        #[arg(long, value_name = "PUBLISHED_FILE_ID")]
        published_file_id: u64,

        /// Path of the file (Pack in Total War) this operation will use.
        #[arg(short, long, required = true, value_name = "PATH")]
        file_path: PathBuf,

        /// Title the mod will receive.
        #[arg(short, long, required = true, value_name = "TITLE")]
        title: String,

        /// Description the mod will receive.
        #[arg(short, long, required = false, value_name = "DESCRIPTION")]
        description: Option<String>,

        /// Tags the mod will receive.
        #[arg(long, required = true, value_name = "COMMA-SEPARATED TAGS")]
        tags: Vec<String>,

        /// Changelog for this specific release.
        #[arg(short, long, required = false, value_name = "CHANGELOG")]
        changelog: Option<String>
    },
}
