//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!Build script for the Workshopper.

/// Windows Build Script.
#[cfg(target_os = "windows")]
fn main() {

    // This lib is in the SteamWorks SDK files. You have to get that somehow in your path.
    println!("cargo:rustc-link-lib=dylib=steam_api64");
}

/// Linux Build Script.
#[cfg(target_os = "linux")]
fn main() {}

/// MacOS Build Script.
#[cfg(target_os = "macos")]
fn main() {}
