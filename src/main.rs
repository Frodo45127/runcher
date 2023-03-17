//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here&: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::cognitive_complexity,           // Disabled due to useless warnings.
    //clippy::cyclomatic_complexity,          // Disabled due to useless warnings.
    clippy::if_same_then_else,              // Disabled because some of the solutions it provides are freaking hard to read.
    clippy::match_bool,                     // Disabled because the solutions it provides are harder to read than the current code.
    clippy::new_ret_no_self,                // Disabled because the reported situations are special cases. So no, I'm not going to rewrite them.
    clippy::suspicious_else_formatting,     // Disabled because the errors it gives are actually false positives due to comments.
    clippy::match_wild_err_arm,             // Disabled because, despite being a bad practice, it's the intended behavior in the code it warns about.
    clippy::clone_on_copy,                  // Disabled because triggers false positives on qt cloning.
    clippy::mutex_atomic,                   // Disabled because in the only instance it triggers, we do it on purpose.
    clippy::too_many_arguments              // Disabled because it gets annoying really quick.
)]

// This disables the terminal window on windows on release builds.
#![windows_subsystem = "windows"]

use qt_widgets::QApplication;

use lazy_static::lazy_static;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use rpfm_lib::games::supported_games::SupportedGames;
use rpfm_lib::integrations::log::*;

use rpfm_ui_common::locale::FALLBACK_LOCALE;
use rpfm_ui_common::settings::*;

use crate::app_ui::AppUI;
use crate::settings_ui::*;

mod actions_ui;
mod app_ui;
mod ffi;
mod integrations;
mod mod_list_ui;
mod pack_list_ui;
mod settings_ui;
mod updater;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// List of supported games and their configuration.
    #[derive(Debug)]
    static ref SUPPORTED_GAMES: SupportedGames = SupportedGames::default();

    /// Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    static ref SENTRY_GUARD: Arc<RwLock<ClientInitGuard>> = Arc::new(RwLock::new(Logger::init(&{
        init_config_path().expect("Error while trying to initialize config path. We're fucked.");
        error_path().unwrap_or_else(|_| PathBuf::from("."))
    }, true, true).unwrap()));
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_SUBTITLE: &str = " -- A New Beginning";

const FALLBACK_LOCALE_EN: &str = include_str!("../locale/English_en.ftl");
const SENTRY_DSN_KEY: &str = "https://4c058b715c304d55b928c3e44a63b4ff@o152833.ingest.sentry.io/4504851217711104";

fn main() {

    // Setup the fallback locale before anything else.
    *FALLBACK_LOCALE.write().unwrap() = FALLBACK_LOCALE_EN.to_string();

    // Setup sentry's dsn for error reporting.
    *SENTRY_DSN.write().unwrap() = SENTRY_DSN_KEY.to_owned();

    // Access the guard to make sure it gets initialized.
    if SENTRY_GUARD.read().unwrap().is_enabled() {
        info!("Sentry Logging support enabled. Starting...");
    } else {
        info!("Sentry Logging support disabled. Starting...");
    }

    // Create the application and start the loop.
    QApplication::init(|_app| {
        match unsafe { AppUI::new() } {
            Ok(app_ui) => {

                // If we closed the window BEFORE executing, exit the app.
                if unsafe { app_ui.main_window().is_visible() } {
                    unsafe { QApplication::exec() }
                } else {
                    0
                }
            }
            Err(error) => {
                error!("{}", error);
                1
            }
        }
    })
}
