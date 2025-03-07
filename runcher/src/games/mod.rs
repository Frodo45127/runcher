//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::{QCheckBox, QDoubleSpinBox, QGridLayout, QSpinBox};

use qt_core::QString;

use anyhow::{anyhow, Result};

use std::fs::File;
use std::io::{BufReader, Read};
#[cfg(target_os = "windows")]use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use rpfm_lib::files::{Container, ContainerPath, FileType};
use rpfm_lib::games::{*, supported_games::*};
use rpfm_lib::utils::files_from_subdir;

use rpfm_ui_common::settings::*;

use crate::app_ui::{AppUI, CUSTOM_MOD_LIST_FILE_NAME};
#[cfg(target_os = "windows")]use crate::mod_manager::integrations::{CREATE_NO_WINDOW, DETACHED_PROCESS};
use crate::SCHEMA;
use crate::settings_ui::{temp_packs_folder, sql_scripts_path};

pub const RESERVED_PACK_NAME: &str = "zzzzzzzzzzzzzzzzzzzzrun_you_fool_thron.pack";
pub const RESERVED_PACK_NAME_ALTERNATIVE: &str = "!!!!!!!!!!!!!!!!!!!!!run_you_fool_thron.pack";

lazy_static::lazy_static! {
    static ref PATCHER_PATH: String = if cfg!(debug_assertions) {
        format!(".\\target\\debug\\{}", PATCHER_EXE)
    } else {
        PATCHER_EXE.to_string()
    };
}

const PATCHER_EXE: &str = "twpatcher.exe";

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub unsafe fn prepare_launch_options(app_ui: &AppUI, game: &GameInfo, data_path: &Path, folder_list: &mut String) -> Result<()> {
    let actions_ui = app_ui.actions_ui();

    // We only use the reserved pack if we need to.
    if (actions_ui.enable_logging_checkbox().is_enabled() && actions_ui.enable_logging_checkbox().is_checked()) ||
        (actions_ui.enable_skip_intro_checkbox().is_enabled() && actions_ui.enable_skip_intro_checkbox().is_checked()) ||
        (actions_ui.remove_trait_limit_checkbox().is_enabled() && actions_ui.remove_trait_limit_checkbox().is_checked()) ||
        (actions_ui.remove_siege_attacker_checkbox().is_enabled() && actions_ui.remove_siege_attacker_checkbox().is_checked()) ||
        (actions_ui.enable_translations_combobox().is_enabled() && actions_ui.enable_translations_combobox().current_index() != 0) ||
        (actions_ui.universal_rebalancer_combobox().is_enabled() && actions_ui.universal_rebalancer_combobox().current_index() != 0) ||
        (actions_ui.enable_dev_only_ui_checkbox().is_enabled() && actions_ui.enable_dev_only_ui_checkbox().is_checked()) ||
        (actions_ui.unit_multiplier_spinbox().is_enabled() && actions_ui.unit_multiplier_spinbox().value() != 1.00) ||
        actions_ui.scripts_to_execute().read().unwrap().iter().any(|(_, item, _)| item.is_checked()) {

        // We need to use an alternative name for Shogun 2, Rome 2, Attila and Thrones because their load order logic for movie packs seems... either different or broken.
        let reserved_pack_name = if game.key() == KEY_SHOGUN_2 || game.key() == KEY_ROME_2 || game.key() == KEY_ATTILA || game.key() == KEY_THRONES_OF_BRITANNIA {
            RESERVED_PACK_NAME_ALTERNATIVE
        } else {
            RESERVED_PACK_NAME
        };

        // If the reserved pack is loaded from a custom folder we need to CLEAR SAID FOLDER before anything else. Otherwise we may end up with old packs messing up stuff.
        if *game.raw_db_version() >= 1 {
            let temp_packs_folder = temp_packs_folder(game)?;
            let files = files_from_subdir(&temp_packs_folder, false)?;
            for file in &files {
                std::fs::remove_file(file)?;
            }
        }

        // Support for add_working_directory seems to be only present in rome 2 and newer games. For older games, we drop the pack into /data.
        let temp_path = if *game.raw_db_version() >= 1 {
            let temp_packs_folder = temp_packs_folder(game)?;
            let temp_path = temp_packs_folder.join(reserved_pack_name);
            folder_list.push_str(&format!("add_working_directory \"{}\";\n", temp_packs_folder.to_string_lossy()));
            temp_path
        } else {
            data_path.join(reserved_pack_name)
        };

        // Prepare the command to generate the temp pack.
        let mut cmd = Command::new(&*PATCHER_PATH);
        cmd.arg("-g");
        cmd.arg(game.key());
        cmd.arg("-l");
        cmd.arg(CUSTOM_MOD_LIST_FILE_NAME);
        cmd.arg("-p");
        cmd.arg(temp_path.to_string_lossy().to_string());   // Use a custom path out of /data, if available.
        cmd.arg("-s");                                      // Skip updates. Updates will be shipped with Runcher updates.

        // Logging check.
        if actions_ui.enable_logging_checkbox().is_enabled() && actions_ui.enable_logging_checkbox().is_checked() {
            cmd.arg("-e");
        }

        // Skip Intros check.
        if actions_ui.enable_skip_intro_checkbox().is_enabled() && actions_ui.enable_skip_intro_checkbox().is_checked() {
            cmd.arg("-i");
        }

        // Remove Trait Limit check.
        if actions_ui.remove_trait_limit_checkbox().is_enabled() && actions_ui.remove_trait_limit_checkbox().is_checked() {
            cmd.arg("-r");
        }

        // Remove Siege Attacker check.
        if actions_ui.remove_siege_attacker_checkbox().is_enabled() && actions_ui.remove_siege_attacker_checkbox().is_checked() {
            cmd.arg("-a");
        }

        // Enable Dev-only UI check.
        if actions_ui.enable_dev_only_ui_checkbox().is_enabled() && actions_ui.enable_dev_only_ui_checkbox().is_checked() {
            cmd.arg("-d");
        }

        // Translations check.
        if actions_ui.enable_translations_combobox().is_enabled() && actions_ui.enable_translations_combobox().current_index() != 0 {
            cmd.arg("-t");
            cmd.arg(app_ui.actions_ui().enable_translations_combobox().current_text().to_std_string());
        }

        // Universal Rebalancer check.
        if actions_ui.universal_rebalancer_combobox().is_enabled() && actions_ui.universal_rebalancer_combobox().current_index() != 0 {
            cmd.arg("-u");
            cmd.arg(app_ui.actions_ui().universal_rebalancer_combobox().current_text().to_std_string());
        }

        // Unit Multiplier check.
        if actions_ui.unit_multiplier_spinbox().is_enabled() && actions_ui.unit_multiplier_spinbox().value() != 1.00 {
            cmd.arg("-m");
            cmd.arg(app_ui.actions_ui().unit_multiplier_spinbox().value().to_string());
        }

        // Script checks.
        let sql_folder = sql_scripts_path()?.join(game.key());
        actions_ui.scripts_to_execute().read().unwrap()
            .iter()
            .filter(|(_, item, _)| item.is_checked())
            .for_each(|(key, item, params)| {
                cmd.arg("--sql-script");

                let script_params = if params.is_empty() {
                    vec![]
                } else {
                    let mut script_params = vec![];
                    let script_container = item.parent_widget().parent_widget();
                    for param in params {
                        let object_name = format!("{}_{}", key, param.0);
                        match &*param.1 {
                            "bool" => {
                                if let Ok(widget) = script_container.find_child::<QCheckBox>(&object_name) {
                                    script_params.push(widget.is_checked().to_string());
                                }
                            },
                            "integer" => {
                                if let Ok(widget) = script_container.find_child::<QSpinBox>(&object_name) {
                                    script_params.push(widget.value().to_string());
                                }
                            },
                            "float" => {
                                if let Ok(widget) = script_container.find_child::<QDoubleSpinBox>(&object_name) {
                                    script_params.push(widget.value().to_string());
                                }
                            },
                            _ => {}
                        }
                    }

                    script_params
                };

                if script_params.is_empty() {
                    cmd.arg(sql_folder.join(format!("{key}.sql")));
                } else {
                    cmd.arg(sql_folder.join(format!("{key}.sql;{}", script_params.join(";"))));
                }
            });

        // This is for creating the terminal window. Without it, the entire process runs in the background and there's no feedback on when it's done.
        #[cfg(target_os = "windows")] if cfg!(debug_assertions) {
            cmd.creation_flags(DETACHED_PROCESS);
        } else {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd.output().map_err(|err| anyhow!("Error when preparing the game patch: {}", err))?;
    }

    Ok(())
}

pub unsafe fn setup_actions(app_ui: &AppUI, game: &GameInfo, game_path: &Path) -> Result<()> {

    // The blockers are needed to avoid issues with game change causing incorrect status to be saved.
    app_ui.actions_ui().play_button().block_signals(true);
    app_ui.actions_ui().copy_load_order_button().block_signals(true);
    app_ui.actions_ui().paste_load_order_button().block_signals(true);
    app_ui.actions_ui().profile_load_button().block_signals(true);
    app_ui.actions_ui().profile_save_button().block_signals(true);
    app_ui.actions_ui().profile_manager_button().block_signals(true);
    app_ui.actions_ui().profile_combobox().block_signals(true);

    app_ui.actions_ui().enable_logging_checkbox().block_signals(true);
    app_ui.actions_ui().enable_skip_intro_checkbox().block_signals(true);
    app_ui.actions_ui().remove_trait_limit_checkbox().block_signals(true);
    app_ui.actions_ui().remove_siege_attacker_checkbox().block_signals(true);
    app_ui.actions_ui().enable_translations_combobox().block_signals(true);
    app_ui.actions_ui().merge_all_mods_checkbox().block_signals(true);
    app_ui.actions_ui().unit_multiplier_spinbox().block_signals(true);
    app_ui.actions_ui().universal_rebalancer_combobox().block_signals(true);
    app_ui.actions_ui().enable_dev_only_ui_checkbox().block_signals(true);
    app_ui.actions_ui().open_game_content_folder().block_signals(true);
    app_ui.actions_ui().save_combobox().block_signals(true);

    // Master check to know if we even have a game path setup correctly.
    let path_is_valid = game_path.exists() && game_path.is_dir() && !game_path.to_string_lossy().is_empty();
    app_ui.actions_ui().play_button().set_enabled(path_is_valid);
    app_ui.actions_ui().copy_load_order_button().set_enabled(path_is_valid);
    app_ui.actions_ui().paste_load_order_button().set_enabled(path_is_valid);
    app_ui.actions_ui().profile_load_button().set_enabled(path_is_valid);
    app_ui.actions_ui().profile_save_button().set_enabled(path_is_valid);
    app_ui.actions_ui().profile_manager_button().set_enabled(path_is_valid);
    app_ui.actions_ui().profile_combobox().set_enabled(path_is_valid);
    app_ui.actions_ui().save_combobox().set_enabled(path_is_valid);

    if path_is_valid {

        // Only set enabled the launch options that work for the current game.
        match game.key() {
            KEY_PHARAOH | KEY_PHARAOH_DYNASTIES => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_WARHAMMER_3 => {
                let schema = SCHEMA.read().unwrap();
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(schema.is_some());
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(schema.is_some());
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_TROY => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_THREE_KINGDOMS => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);    // 3K doesn't support logging by default.
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_WARHAMMER_2 => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_WARHAMMER => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);    // Warhammer 1 doesn't support logging by default.
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_THRONES_OF_BRITANNIA => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_ATTILA => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_ROME_2 => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(true);
            },
            KEY_SHOGUN_2 => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(true);
                app_ui.actions_ui().save_combobox().set_enabled(false);
            },
            KEY_NAPOLEON => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(false);
                app_ui.actions_ui().save_combobox().set_enabled(false);
            },
            KEY_EMPIRE => {
                app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().remove_trait_limit_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().remove_siege_attacker_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
                app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().universal_rebalancer_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().enable_dev_only_ui_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
                app_ui.actions_ui().open_game_content_folder().set_enabled(false);
                app_ui.actions_ui().save_combobox().set_enabled(false);
            }
            &_ => {},
        }

        // Disable this until I figure out how to fix the performance problems, and I change the pack to be on /data
        app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);

        // Update the launch options for the new game.
        app_ui.actions_ui().enable_logging_checkbox().set_checked(setting_bool(&format!("enable_logging_{}", game.key())));
        app_ui.actions_ui().enable_skip_intro_checkbox().set_checked(setting_bool(&format!("enable_skip_intros_{}", game.key())));
        app_ui.actions_ui().remove_trait_limit_checkbox().set_checked(setting_bool(&format!("remove_trait_limit_{}", game.key())));
        app_ui.actions_ui().remove_siege_attacker_checkbox().set_checked(setting_bool(&format!("remove_siege_attacker_{}", game.key())));
        app_ui.actions_ui().merge_all_mods_checkbox().set_checked(setting_bool(&format!("merge_all_mods_{}", game.key())));
        app_ui.actions_ui().enable_dev_only_ui_checkbox().set_checked(setting_bool(&format!("enable_dev_only_ui_{}", game.key())));
        app_ui.actions_ui().unit_multiplier_spinbox().set_value({
            let value = setting_f32(&format!("unit_multiplier_{}", game.key()));
            if value == 0.00 {
                1.00
            } else {
                value
            }
        } as f64);

        // Populate the list of translations depending on what local_XX packs the game has.
        app_ui.actions_ui().enable_translations_combobox().clear();
        app_ui.actions_ui().enable_translations_combobox().insert_item_int_q_string(0, &QString::from_std_str("--"));
        app_ui.actions_ui().enable_translations_combobox().set_current_index(0);

        if let Ok(ca_packs) = game.ca_packs_paths(game_path) {
            let mut languages = ca_packs.iter()
                .filter_map(|path| path.file_stem())
                .filter(|name| name.to_string_lossy().starts_with("local_"))
                .map(|name| name.to_string_lossy().split_at(6).1.to_uppercase())
                .collect::<Vec<_>>();

            // Sort, and remove anything longer than 2 characters to avoid duplicates.
            languages.retain(|lang| lang.chars().count() == 2);
            languages.sort();

            for (index, language) in languages.iter().enumerate() {
                app_ui.actions_ui().enable_translations_combobox().insert_item_int_q_string(index as i32 + 1, &QString::from_std_str(language));
            }

            let language_to_select = setting_string(&format!("enable_translations_{}", game.key()));
            app_ui.actions_ui().enable_translations_combobox().set_current_text(&QString::from_std_str(language_to_select));
        }

        // Populate the list of mods to rebalance over.
        app_ui.actions_ui().universal_rebalancer_combobox().clear();
        app_ui.actions_ui().universal_rebalancer_combobox().insert_item_int_q_string(0, &QString::from_std_str("--"));
        app_ui.actions_ui().universal_rebalancer_combobox().set_current_index(0);

        // We need to find all enabled packs with a copy of land_units
        let mut load_order = app_ui.game_load_order().read().unwrap().clone();
        if let Ok(game_data_path) = game.data_path(game_path) {
            if let Some(ref game_config) = *app_ui.game_config().read().unwrap() {
                load_order.update(game_config, &game_data_path);

                let mut packs_for_rebalancer = load_order.packs().iter()
                    .filter_map(|(key, pack)| {
                        if !pack.files_by_type_and_paths(&[FileType::DB], &[ContainerPath::Folder("db/land_units_tables/".to_owned())], true).is_empty() {
                            Some(key)
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>();

                packs_for_rebalancer.sort();
                for pack in &packs_for_rebalancer {
                    app_ui.actions_ui().universal_rebalancer_combobox().add_item_q_string(&QString::from_std_str(pack));
                }

                // Only apply it if it's still valid.
                let pack_to_select = setting_string(&format!("universal_rebalancer_{}", game.key()));
                if app_ui.actions_ui().universal_rebalancer_combobox().find_text_1a(&QString::from_std_str(&pack_to_select)) != -1 {
                    app_ui.actions_ui().universal_rebalancer_combobox().set_current_text(&QString::from_std_str(&pack_to_select));
                }
            }
        }
    }

    // Unblock all blocked signals.
    app_ui.actions_ui().play_button().block_signals(false);
    app_ui.actions_ui().copy_load_order_button().block_signals(false);
    app_ui.actions_ui().paste_load_order_button().block_signals(false);
    app_ui.actions_ui().profile_load_button().block_signals(false);
    app_ui.actions_ui().profile_save_button().block_signals(false);
    app_ui.actions_ui().profile_manager_button().block_signals(false);
    app_ui.actions_ui().profile_combobox().block_signals(false);

    app_ui.actions_ui().enable_logging_checkbox().block_signals(false);
    app_ui.actions_ui().enable_skip_intro_checkbox().block_signals(false);
    app_ui.actions_ui().remove_trait_limit_checkbox().block_signals(false);
    app_ui.actions_ui().remove_siege_attacker_checkbox().block_signals(false);
    app_ui.actions_ui().enable_translations_combobox().block_signals(false);
    app_ui.actions_ui().merge_all_mods_checkbox().block_signals(false);
    app_ui.actions_ui().unit_multiplier_spinbox().block_signals(false);
    app_ui.actions_ui().universal_rebalancer_combobox().block_signals(false);
    app_ui.actions_ui().enable_dev_only_ui_checkbox().block_signals(false);
    app_ui.actions_ui().save_combobox().block_signals(false);
    app_ui.actions_ui().open_game_content_folder().block_signals(false);

    // Scripts are done in a separate step, because they're dynamic.
    {
        let script_parent = app_ui.actions_ui().scripts_container();
        let script_layout = script_parent.layout();

        if !script_layout.is_null() {
            let script_layout = script_layout.static_downcast::<QGridLayout>();
            loop {
                let item = script_layout.take_at(0);
                if !item.is_null() {
                    item.widget().delete_later();
                } else {
                    break;
                }
            }
        }

        let sql_script_paths = files_from_subdir(&sql_scripts_path()?.join(game.key()), false)?;
        let mut script_items = app_ui.actions_ui().scripts_to_execute().write().unwrap();
        script_items.clear();

        for path in sql_script_paths {
            if let Ok(script_info) = get_script_info(&path) {
                let script_item = app_ui.actions_ui().new_launch_script_option(game.key(), "autocorrection", &script_info);
                script_items.push((script_info.0, script_item, script_info.2));
            }
        }
    }

    Ok(())
}

fn get_script_info(path: &Path) -> Result<(String, String, Vec<(String, String, String, String)>)> {
    let mut param_list = vec![];
    let mut file = BufReader::new(File::open(path)?);
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let script_key = path.file_stem().unwrap().to_string_lossy().to_string();
    let pretty_name = match data.lines().find(|x| x.starts_with("-- Pretty name:")) {
        Some(line) => line[15..].to_owned(),
        None => script_key.clone(),
    };

    let start_pos = data.find("-- Parameters:");
    let end_pos = data.find("-- End of parameters.");

    if let Some(start_pos) = start_pos {
        if let Some(end_pos) = end_pos {
            if start_pos < end_pos {
                let params = data[start_pos + 14..end_pos]
                    .replace("-- ", "")
                    .replace("\r\n", "\n");

                let params_split = params.split("\n")
                    .map(|x| x.trim())
                    .filter(|x| !x.is_empty())
                    .map(|x| x.split(":").collect::<Vec<_>>())
                    .filter(|x| x.len() == 4)
                    .collect::<Vec<_>>();

                for param in &params_split {
                    let param_id = param[0];
                    let param_type = param[1];
                    let param_default = param[2];
                    let param_name = param[3];

                    param_list.push((param_id.to_owned(), param_type.to_owned(), param_default.to_owned(), param_name.to_owned()));
                }
            }
        }
    }

    Ok((script_key, pretty_name, param_list))
}
