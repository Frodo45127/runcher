//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;
use base64::{Engine, prelude::BASE64_STANDARD};
use execute_command::ExecuteCommand;
use interprocess::local_socket::LocalSocketStream;
use steamworks::Client;

use std::io::Write;
#[cfg(target_os = "windows")] use std::os::windows::process::CommandExt;
use std::process::Command;

use rpfm_lib::integrations::log::info;

pub mod ugc;

#[cfg(target_os = "windows")] const CREATE_NO_WINDOW: u32 = 0x08000000;
const IPC_NAME_GET_STEAM_USER_ID: &str = "runcher_get_steam_user_id";

//---------------------------------------------------------------------------//
//                        Generic public functions
//---------------------------------------------------------------------------//

/// This function is used to launch games with the Steam API enabled.
///
/// command is expected to be the full command to launch the game as a Rust std::process::Command.
pub fn launch_game(base64: bool, steam_id: u32, command: &str) -> Result<()> {

    // If we're in base64 mode, decode the args.
    let command = if base64 {
        String::from_utf8(BASE64_STANDARD.decode(command)?)?
    } else {
        command.to_owned()
    };

    // Start the api.
    //
    // We really just need the API running when launching the exe, don't need to call the api for anything else.
    let _client = Client::init_app(steam_id)?;

    // Launch the game.
    let mut game_command = Command::parse(command)?;

    // This disables the terminal when executing the command.
    #[cfg(target_os = "windows")]game_command.creation_flags(CREATE_NO_WINDOW);
    let mut handle = game_command.spawn()?;
    let _ = handle.wait()?;

    Ok(())
}

pub fn user_id(steam_id: u32) -> Result<()> {
    let (client, _) = Client::init_app(steam_id)?;
    let steam_user_id = client.user().steam_id();

    info!("User Steam ID: {}", steam_user_id.raw());

    if let Ok(mut stream) = LocalSocketStream::connect(IPC_NAME_GET_STEAM_USER_ID) {
        let _ = stream.write(&steam_user_id.raw().to_le_bytes());
    }

    Ok(())
}
