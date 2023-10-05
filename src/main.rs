//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here&: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
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

use qt_gui::QColor;
use qt_gui::QGuiApplication;
use qt_gui::{QPalette, q_palette::{ColorRole, ColorGroup}};

use qt_core::QCoreApplication;
use qt_core::QString;

use lazy_static::lazy_static;

use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicPtr, RwLock};
use std::thread;

use rpfm_lib::games::supported_games::SupportedGames;
use rpfm_lib::integrations::log::*;
use rpfm_lib::schema::Schema;

use rpfm_ui_common::locale::FALLBACK_LOCALE;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::app_ui::AppUI;
use crate::communications::*;
use crate::settings_ui::*;

mod actions_ui;
mod app_ui;
mod background_thread;
mod communications;
mod ffi;
mod games;
mod mod_manager;
mod mod_list_ui;
mod network_thread;
mod pack_list_ui;
mod settings_ui;
mod updater_ui;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// List of supported games and their configuration.
    #[derive(Debug)]
    static ref SUPPORTED_GAMES: SupportedGames = SupportedGames::default();

    /// Currently loaded schema.
    static ref SCHEMA: Arc<RwLock<Option<Schema>>> = Arc::new(RwLock::new(None));

    /// Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    static ref SENTRY_GUARD: Arc<RwLock<ClientInitGuard>> = Arc::new(RwLock::new(Logger::init(&{
        init_config_path().expect("Error while trying to initialize config path. We're fucked.");
        error_path().unwrap_or_else(|_| PathBuf::from("."))
    }, true, true).unwrap()));

    /// Light stylesheet.
    static ref LIGHT_STYLE_SHEET: AtomicPtr<QString> = unsafe {
        let app = QCoreApplication::instance();
        let qapp = app.static_downcast::<QApplication>();
        atomic_from_cpp_box(qapp.style_sheet())
    };

    /// Bright and dark palettes of colours for Windows.
    /// The dark one is taken from here, with some modifications: https://gist.github.com/QuantumCD/6245215
    static ref LIGHT_PALETTE: AtomicPtr<QPalette> = unsafe { atomic_from_cpp_box(QPalette::new()) };
    static ref DARK_PALETTE: AtomicPtr<QPalette> = unsafe {{
        let palette = QPalette::new();

        // Base config.
        palette.set_color_2a(ColorRole::Window, &QColor::from_3_int(51, 51, 51));
        palette.set_color_2a(ColorRole::WindowText, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::Base, &QColor::from_3_int(34, 34, 34));
        palette.set_color_2a(ColorRole::AlternateBase, &QColor::from_3_int(51, 51, 51));
        palette.set_color_2a(ColorRole::ToolTipBase, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::ToolTipText, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::Text, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::Button, &QColor::from_3_int(51, 51, 51));
        palette.set_color_2a(ColorRole::ButtonText, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::BrightText, &QColor::from_3_int(255, 0, 0));
        palette.set_color_2a(ColorRole::Link, &QColor::from_3_int(42, 130, 218));
        palette.set_color_2a(ColorRole::Highlight, &QColor::from_3_int(42, 130, 218));
        palette.set_color_2a(ColorRole::HighlightedText, &QColor::from_3_int(204, 204, 204));

        // Disabled config.
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Window, &QColor::from_3_int(34, 34, 34));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::WindowText, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Base, &QColor::from_3_int(34, 34, 34));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::AlternateBase, &QColor::from_3_int(34, 34, 34));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::ToolTipBase, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::ToolTipText, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Text, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Button, &QColor::from_3_int(34, 34, 34));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::ButtonText, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::BrightText, &QColor::from_3_int(170, 0, 0));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Link, &QColor::from_3_int(42, 130, 218));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Highlight, &QColor::from_3_int(42, 130, 218));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::HighlightedText, &QColor::from_3_int(85, 85, 85));

        atomic_from_cpp_box(palette)
    }};

    /// Global variable to hold the sender/receivers used to comunicate between threads.
    static ref CENTRAL_COMMAND: CentralCommand<Response> = CentralCommand::default();
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_SUBTITLE: &str = " -- When I learned maths";

const GITHUB_URL: &str = "https://github.com/Frodo45127/runcher";
const DISCORD_URL: &str = "https://discord.gg/moddingden";
const PATREON_URL: &str = "https://www.patreon.com/RPFM";

const FALLBACK_LOCALE_EN: &str = include_str!("../locale/English_en.ftl");
const SENTRY_DSN_KEY: &str = "https://4c058b715c304d55b928c3e44a63b4ff@o152833.ingest.sentry.io/4504851217711104";

fn main() {

    // This needs to be initialised before anything else.
    unsafe {

        // Settings stuff.
        QCoreApplication::set_organization_domain(&QString::from_std_str("com"));
        QCoreApplication::set_organization_name(&QString::from_std_str("FrodoWazEre"));
        QCoreApplication::set_application_name(&QString::from_std_str("runcher"));

        // This fixes the app icon on wayland.
        QGuiApplication::set_desktop_file_name(&QString::from_std_str("runcher"));
    }

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

    // Create the background and network threads, where all the magic will happen.
    info!("Initializing threads...");
    let bac_handle = thread::spawn(|| { background_thread::background_loop(); });
    let net_handle = thread::spawn(|| { network_thread::network_loop(); });

    // Create the application and start the loop.
    QApplication::init(|_app| {
        match unsafe { AppUI::new() } {
            Ok(app_ui) => {

                // If we closed the window BEFORE executing, exit the app.
                let exit_code = if unsafe { app_ui.main_window().is_visible() } {
                    unsafe { QApplication::exec() }
                } else { 0 };

                // Close and rejoin the threads on exit, so we don't leave a rogue thread running.
                CENTRAL_COMMAND.send_background(Command::Exit);
                CENTRAL_COMMAND.send_network(Command::Exit);

                let _ = bac_handle.join();
                let _ = net_handle.join();

                exit_code
            }
            Err(error) => {
                error!("{}", error);

                // Close and rejoin the threads on exit, so we don't leave a rogue thread running.
                CENTRAL_COMMAND.send_background(Command::Exit);
                CENTRAL_COMMAND.send_network(Command::Exit);

                let _ = bac_handle.join();
                let _ = net_handle.join();

                55
            }
        }
    })
}
