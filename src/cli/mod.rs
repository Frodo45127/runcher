//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the code to parse cli arguments, for automation.

use anyhow::Result;
use clap::{builder::PossibleValuesParser, Parser};

use rpfm_lib::games::supported_games::*;
use rpfm_lib::integrations::log::*;

use rpfm_ui_common::settings::setting_string;
use rpfm_ui_common::utils::log_to_status_bar;

use crate::app_ui::AppUI;

//---------------------------------------------------------------------------//
//                          Struct/Enum Definitions
//---------------------------------------------------------------------------//

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {

    /// Game we we want to start with. Overrides default game.
    #[arg(short, long, required = false, value_name = "GAME", value_parser = PossibleValuesParser::new(game_keys()))]
    game: Option<String>,

    /// Profile to start with. Requires a game.
    #[arg(short, long, required = false, value_name = "PROFILE_NAME")]
    profile: Option<String>,

    /// If we should autostart the game/profile combo. Skips the UI. Requires a game, profile is optional.
    #[arg(short, long, required = false)]
    autostart: bool,
}

/// Function to get the supported game keys.
fn game_keys() -> Vec<&'static str> {
    let supported_games = SupportedGames::default();
    supported_games.game_keys_sorted().to_vec()
}

impl Cli {

    pub unsafe fn parse_args(app_ui: &AppUI) -> Result<bool> {

        // Parse the entire cli command.
        let cli = Self::parse();

        // If we're not autostarting, make the main window visible, then trigger an event loop cycle
        // so the window is shown, then we do the expensive stuff.
        if !cli.autostart {
            app_ui.main_window().show();
            app_ui.toggle_main_window(false);

            log_to_status_bar(app_ui.main_window().status_bar(), "Initializing, please wait...");
            let event_loop = qt_core::QEventLoop::new_0a();
            event_loop.process_events_0a();
        }

        // Game override.
        let mut game_passed = false;
        let mut default_game = setting_string("default_game");
        match cli.game {
            Some(ref game) => {

                // Set the game selected based on the default game. If we passed a game through an argument, use that one.
                //
                // Note: set_checked does *NOT* trigger the slot for changing game selected. We need to trigger that one manually.
                match &**game {
                    KEY_PHARAOH |
                    KEY_WARHAMMER_3 |
                    KEY_TROY |
                    KEY_THREE_KINGDOMS |
                    KEY_WARHAMMER_2 |
                    KEY_WARHAMMER |
                    KEY_THRONES_OF_BRITANNIA |
                    KEY_ATTILA |
                    KEY_ROME_2 |
                    KEY_SHOGUN_2 |
                    KEY_NAPOLEON |
                    KEY_EMPIRE => {
                        info!("Valid game provided through arg, using {} as default game.", game);
                        default_game = game.to_owned();
                        game_passed = true;
                    },
                    _ => info!("Invalid game provided through arg (\"{}\"), using {} as default game.", game, default_game),
                }
            }
            None => info!("No default game provided through arg, using {} as default game.", default_game),
        }

        // Set the default game, and set it in the UI too.
        match &*default_game {
            KEY_PHARAOH => app_ui.game_selected_pharaoh().set_checked(true),
            KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3().set_checked(true),
            KEY_TROY => app_ui.game_selected_troy().set_checked(true),
            KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms().set_checked(true),
            KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2().set_checked(true),
            KEY_WARHAMMER => app_ui.game_selected_warhammer().set_checked(true),
            KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia().set_checked(true),
            KEY_ATTILA => app_ui.game_selected_attila().set_checked(true),
            KEY_ROME_2 => app_ui.game_selected_rome_2().set_checked(true),
            KEY_SHOGUN_2 => app_ui.game_selected_shogun_2().set_checked(true),
            KEY_NAPOLEON => app_ui.game_selected_napoleon().set_checked(true),
            KEY_EMPIRE => app_ui.game_selected_empire().set_checked(true),
            _ => app_ui.game_selected_warhammer_3().set_checked(true),
        }

        // This may fail for path problems.
        //
        // Also, the game we already have loaded is arena. We don't need to force a manual reload with that one.
        app_ui.change_game_selected(false)?;

        // If we're not autostarting, enable the UI here.
        if !cli.autostart {
            app_ui.toggle_main_window(true);
        }

        // NOTE: This is a ñapa until the profile rework is done.
        if game_passed {

            // Default profile. Only check if we have a valid game, because this needs the game to be set.
            match cli.profile {
                Some(ref profile) => {
                    info!("Profile {} provided through args.", profile);

                    match app_ui.load_profile(Some(profile.to_string())) {
                        Ok(_) => info!("Profile loaded correctly."),
                        Err(error) => error!("Error loading profile {}: {}.", profile, error),
                    }
                },
                None => info!("No profile provided through arg."),
            }

            // Autostart skipping ui? Only with game loaded, and last.
            if cli.autostart {
                info!("Autostart provided. Skipping UI and loading the game.");
                app_ui.launch_game()?;
                return Ok(true);
            } else {
                info!("Autostart not provided, or provided as false.");
            }
        } else {
            info!("No valid game provided through args. Ignoring subsequent checks.");
        }

        Ok(false)
    }
}
