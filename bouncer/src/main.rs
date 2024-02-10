//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here&: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This disables the terminal window on windows on release builds.
#![windows_subsystem = "windows"]

use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() {

    let exe_name = "shogun2.exe";
    let mod_file_name = "mod_list.txt";
    let mut game_root_folder = std::env::current_exe().unwrap();
    game_root_folder.pop();
    game_root_folder.pop();

    let mut command = Command::new("cmd");
    command.arg("/C");
    command.arg("start");
    command.arg("/d");
    command.arg(game_root_folder.to_string_lossy().replace('\\', "/"));
    command.arg(exe_name);
    command.arg(mod_file_name.to_owned() + ";");

    // This disables the terminal when executing the command.
    #[cfg(target_os = "windows")]command.creation_flags(CREATE_NO_WINDOW);
    command.output().unwrap();
}
