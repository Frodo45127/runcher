//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::{QCheckBox, QComboBox, QDoubleSpinBox, QGridLayout, QSpinBox};

use qt_gui::QResizeEvent;

use qt_core::{QSize, QString};

use anyhow::{anyhow, Result};

use std::cell::LazyCell;
use std::collections::HashMap;
#[cfg(target_os = "windows")]use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use common_utils::sql::{ParamType, Preset, SQLScript};

use rpfm_lib::files::{Container, ContainerPath, FileType};
use rpfm_lib::games::{*, supported_games::*};
use rpfm_lib::utils::files_from_subdir;

use rpfm_ui_common::settings::*;

use crate::app_ui::{AppUI, CUSTOM_MOD_LIST_FILE_NAME};
use crate::mod_manager::game_config::GameConfig;
#[cfg(target_os = "windows")]use crate::mod_manager::integrations::DETACHED_PROCESS;
use crate::mod_manager::load_order::LoadOrder;
use crate::SCHEMA;
use crate::settings_ui::{temp_packs_folder, sql_scripts_extracted_path, sql_scripts_extracted_extended_path, sql_scripts_local_path, sql_scripts_remote_path};

pub const RESERVED_PACK_NAME: &str = "zzzzzzzzzzzzzzzzzzzzrun_you_fool_thron.pack";
pub const RESERVED_PACK_NAME_ALTERNATIVE: &str = "!!!!!!!!!!!!!!!!!!!!!run_you_fool_thron.pack";

const PATCHER_PATH: LazyCell<String> = LazyCell::new(|| {
    let base_path = std::env::current_dir().unwrap();
    let base_path = base_path.display();
    if cfg!(debug_assertions) {
        format!("{}/target/debug/{}", base_path, PATCHER_EXE)
    } else {
        format!("{}/{}", base_path, PATCHER_EXE)
    }
});

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
        actions_ui.scripts_to_execute().read().unwrap().iter().any(|(_, item)| item.is_checked()) {

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
        let mut cmd = Command::new("cmd");
        cmd.arg("/C");
        cmd.arg(&*PATCHER_PATH);
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
        let sql_folder_extracted = sql_scripts_extracted_extended_path()?;
        let sql_folder_local = sql_scripts_local_path()?.join(game.key());
        let sql_folder_remote = sql_scripts_remote_path()?.join(game.key());
        actions_ui.scripts_to_execute().read().unwrap()
            .iter()
            .filter(|(_, item)| item.is_checked())
            .for_each(|(script, item)| {
                cmd.arg("--sql-script");

                let script_params = if script.metadata().parameters().is_empty() {
                    vec![]
                } else {
                    let mut script_params = vec![];
                    let script_container = item.parent_widget().parent_widget();

                    // First check if we have a preset set. If not, we can check each param.
                    let preset_combo_name = format!("{}_preset_combo", script.metadata().key());
                    let preset_key = if let Ok(widget) = script_container.find_child::<QComboBox>(&preset_combo_name) {
                        widget.current_text().to_std_string()
                    } else {
                        String::new()
                    };

                    let preset = if !preset_key.is_empty() {
                        let preset_path = sql_scripts_extracted_path().unwrap().join("twpatcher/presets");
                        if preset_path.is_dir() {
                            files_from_subdir(&preset_path, false).unwrap()
                                .iter()
                                .filter_map(|x| Preset::read(x).ok())
                                .find(|x| *x.key() == preset_key)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    match preset {
                        Some(preset) => {
                            for param in script.metadata().parameters() {
                                match preset.params().get(param.key()) {
                                    Some(value) => script_params.push(value.to_string()),
                                    None => script_params.push(param.default_value().to_string()),
                                }
                            }
                        }
                        None => {
                            for param in script.metadata().parameters() {
                                let object_name = format!("{}_{}", script.metadata().key(), param.key());
                                match param.r#type() {
                                    ParamType::Bool => {
                                        if let Ok(widget) = script_container.find_child::<QCheckBox>(&object_name) {
                                            script_params.push(widget.is_checked().to_string());
                                        }
                                    },
                                    ParamType::Integer => {
                                        if let Ok(widget) = script_container.find_child::<QSpinBox>(&object_name) {
                                            script_params.push(widget.value().to_string());
                                        }
                                    },
                                    ParamType::Float => {
                                        if let Ok(widget) = script_container.find_child::<QDoubleSpinBox>(&object_name) {
                                            script_params.push(widget.value().to_string());
                                        }
                                    },
                                }
                            }
                        }
                    }


                    script_params
                };

                // When there's a collision, default to the local script path.
                let script_name = format!("{}.yml", script.metadata().key());
                let local_script_path = sql_folder_local.join(&script_name);
                let extracted_script_path = sql_folder_extracted.join(&script_name);
                let remote_script_path = sql_folder_remote.join(&script_name);
                let script_path = if PathBuf::from(&local_script_path).is_file() {
                    local_script_path
                } else if PathBuf::from(&extracted_script_path).is_file() {
                    extracted_script_path
                } else {
                    remote_script_path
                };

                if script_params.is_empty() {
                    cmd.arg(script_path);
                } else {
                    cmd.arg(format!("{};{}", script_path.to_string_lossy().to_string().replace("\\", "/"), script_params.join(";")));
                }
            });

        cmd.creation_flags(DETACHED_PROCESS);

        let h = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn().map_err(|err| anyhow!("Error when preparing the game patch: {}", err))?;

        if let Ok(output) = h.wait_with_output() {
            if !output.status.success() {
                let out = String::from_utf8(output.stdout)?;
                let err = String::from_utf8(output.stderr)?;
                return Err(anyhow!("Something failed while creating the load order patch. Check the patcher terminal to see what happened. Specifically, this: \n\n{err}\n\nHere's the rest of the output: \n\n{out}"))
            }
        }
    }

    Ok(())
}

pub unsafe fn setup_actions(app_ui: &AppUI, game: &GameInfo, game_config: &GameConfig, game_path: &Path, load_order: &LoadOrder) -> Result<()> {

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
        let mut load_order = load_order.clone();
        if let Ok(game_data_path) = game.data_path(game_path) {
            load_order.update(game_config, game, &game_data_path);

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

    // Scripts are done in a separate step, because they're dynamic. Priority is:
    // - Local scripts.
    // - Extracted scripts.
    // - Remote scripts.
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

        let extracted_folder = sql_scripts_extracted_path()?;
        let extracted_scripts_folder = extracted_folder.join("twpatcher/scripts");
        let presets_folder = extracted_folder.join("twpatcher/presets");

        let local_folder = sql_scripts_local_path()?;
        let remote_folder = sql_scripts_remote_path()?;
        let mut sql_script_paths = files_from_subdir(&local_folder.join(game.key()), false)?;

        // Only add extracted paths if they don't collide with local paths, as local paths take priority.
        if let Ok(extracted_files) = files_from_subdir(&extracted_scripts_folder, false) {
            for extracted_file in &extracted_files {
                if let Ok(relative_path) = extracted_file.strip_prefix(&extracted_scripts_folder) {
                    if !local_folder.join(relative_path).is_file() {
                        sql_script_paths.push(extracted_file.to_path_buf());
                    }
                }
            }
        }

        // Only add remote paths if they don't collide with local or extracted paths, as they take priority.
        if let Ok(remote_files) = files_from_subdir(&remote_folder.join(game.key()), false) {
            for remote_file in &remote_files {
                if let Ok(relative_path) = remote_file.strip_prefix(&remote_folder) {
                    if !local_folder.join(relative_path).is_file() && !extracted_scripts_folder.join(relative_path).is_file() {
                        sql_script_paths.push(remote_file.to_path_buf());
                    }
                }
            }
        }

        let presets = files_from_subdir(&presets_folder, false).unwrap_or_default()
            .iter()
            .filter_map(|x| Preset::read(x).ok())
            .collect::<Vec<_>>();

        let mut presets_by_script: HashMap<String, Vec<Preset>> = HashMap::new();
        for preset in &presets {
            match presets_by_script.get_mut(preset.script_key()) {
                Some(presets) => presets.push(preset.clone()),
                None => { presets_by_script.insert(preset.script_key().to_owned(), vec![preset.clone()]);},
            }
        }

        let mut script_items = app_ui.actions_ui().scripts_to_execute().write().unwrap();
        script_items.clear();

        for path in sql_script_paths {
            if let Some(extension) = path.extension() {

                // Only load yml files.
                if extension == "yml" {
                    if let Ok(script) = SQLScript::from_path(&path) {
                        let presets = presets_by_script.get(script.metadata().key()).cloned().unwrap_or_else(|| vec![]);
                        let script_item = app_ui.actions_ui().new_launch_script_option(game.key(), "autocorrection", &script, &presets);
                        script_items.push((script, script_item));
                    }
                }
            }
        }

        // Trigger a resize of the menu, so it's not compressed.
        let menu = app_ui.actions_ui().play_button().menu();
        let event = QResizeEvent::new(&QSize::new_0a(), &menu.size());
        qt_core::QCoreApplication::send_event(menu, &event);
    }

    Ok(())
}
