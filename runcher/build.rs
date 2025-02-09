//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!Build script for the Runcher.
//!
//!Here it goes all linking/cross-language compilation/platform-specific stuff that's needed in order to compile the Runcher.

#[cfg(target_os = "windows")] use std::fs::{copy, DirBuilder};
use std::io::{stderr, stdout, Write};
use std::process::{Command, exit};

/// Windows Build Script.
#[cfg(target_os = "windows")]
fn main() {
    common_config();

    // Copy the dll needed for workshopper.
    std::fs::copy("./../workshopper/3rdparty/steam_api64.dll", "./../target/debug/steam_api64.dll").unwrap();

    // This compiles the custom widgets lib.
    match Command::new("nmake").current_dir("./../3rdparty/src/qt_runcher_extensions/").output() {
        Ok(output) => {
            stdout().write_all(&output.stdout).unwrap();
            stderr().write_all(&output.stderr).unwrap();
        }
        Err(error) => {
            stdout().write_all(error.to_string().as_bytes()).unwrap();
            stdout().write_all(b"ERROR: You either don't have nmake installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
            exit(99);
        }
    }

    // Icon/Exe info gets added here.
    let mut res = winres::WindowsResource::new();
    res.set_icon("./../icons/runcher.ico");
    res.set("LegalCopyright","Copyright (c) - Ismael Gutiérrez González");
    res.set("ProductName","Runcher");
    if let Err(error) = res.compile() { println!("Error: {}", error); }

    // Copy the icon theme so it can be accessed by debug builds.
    DirBuilder::new().recursive(true).create("./../target/debug/data/icons/breeze/").unwrap();
    DirBuilder::new().recursive(true).create("./../target/debug/data/icons/breeze-dark/").unwrap();
    copy("./../icons/breeze-icons.rcc", "./../target/debug/data/icons/breeze/breeze-icons.rcc").unwrap();
    copy("./../icons/breeze-icons-dark.rcc", "./../target/debug/data/icons/breeze-dark/breeze-icons-dark.rcc").unwrap();
}

/// Linux Build Script.
#[cfg(target_os = "linux")]
fn main() {
    common_config();

    // This compiles the custom widgets lib.
    match Command::new("make").current_dir("./../3rdparty/src/qt_runcher_extensions/").output() {
        Ok(output) => {
            stdout().write_all(&output.stdout).unwrap();
            stderr().write_all(&output.stderr).unwrap();
        }
        Err(error) => {
            stdout().write_all(error.to_string().as_bytes()).unwrap();
            stdout().write_all(b"ERROR: You either don't have make installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
            exit(99);
        }
    }
}

/// MacOS Build Script.
#[cfg(target_os = "macos")]
fn main() {
    common_config();

    // This compiles the custom widgets lib.
    match Command::new("gmake").current_dir("./../3rdparty/src/qt_runcher_extensions/").output() {
        Ok(output) => {
            stdout().write_all(&output.stdout).unwrap();
            stderr().write_all(&output.stderr).unwrap();
        }
        Err(error) => {
            stdout().write_all(error.to_string().as_bytes()).unwrap();
            stdout().write_all(b"ERROR: You either don't have gmake installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
            exit(99);
        }
    }
}

/// This function defines common configuration stuff for all platforms.
fn common_config() {

    // This is to make RPFM able to see the extra libs we need while building.
    println!("cargo:rustc-link-search=native=./3rdparty/builds");
    println!("cargo:rustc-link-lib=dylib=qt_runcher_extensions");
    println!("cargo:rustc-link-lib=dylib=KF5Completion");
    println!("cargo:rustc-link-lib=dylib=KF5IconThemes");
    println!("cargo:rustc-link-lib=dylib=KF5TextEditor");
    println!("cargo:rustc-link-lib=dylib=KF5XmlGui");
    println!("cargo:rustc-link-lib=dylib=KF5WidgetsAddons");

    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=./3rdparty/builds/*");
    println!("cargo:rerun-if-changed=./3rdparty/src/qt_runcher_extensions/*");
    println!("cargo:rerun-if-changed=./runcher/build.rs");

    // This creates the makefile for the custom widget lib.
    match Command::new("qmake")
        .arg("-o")
        .arg("Makefile")
        .arg("qt_runcher_extensions.pro")
        .current_dir("./../3rdparty/src/qt_runcher_extensions/").output() {
        Ok(output) => {
            stdout().write_all(&output.stdout).unwrap();
            stderr().write_all(&output.stderr).unwrap();
        }
        Err(error) => {
            stdout().write_all(error.to_string().as_bytes()).unwrap();
            stdout().write_all(b"ERROR: You either don't have qmake installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
            exit(99);
        }
    }
}
