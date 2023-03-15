//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::SlotNoArgs;

use getset::*;
use qt_widgets::QMainWindow;

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use rpfm_ui_common::clone;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::show_dialog;

use crate::settings_ui::init_settings;
use crate::settings_ui::SettingsUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct SettingsUISlots {
    restore_default: QBox<SlotNoArgs>,
    select_game_paths: BTreeMap<String, QBox<SlotNoArgs>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl SettingsUISlots {

    pub unsafe fn new(ui: &Arc<SettingsUI>, main_window: QPtr<QMainWindow>) -> Self {
        let restore_default = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {

                // Restore all settings and reload the view, WITHOUT SAVING THE SETTINGS.
                // An exception are the original states. We need to keep those.
                let q_settings = settings();
                let keys = q_settings.all_keys();

                let mut old_settings = HashMap::new();
                for i in 0..keys.count_0a() {
                    old_settings.insert(keys.at(i).to_std_string(), setting_variant_from_q_setting(&q_settings, &keys.at(i).to_std_string()));
                }

                // Fonts are a bit special. Init picks them up from the running app, not from a fixed value,
                // so we need to manually overwrite them here before init_settings gets triggered.
                //let original_font_name = setting_string("original_font_name");
                //let original_font_size = setting_int("original_font_size");

                q_settings.clear();

                //set_setting_string_to_q_setting(&q_settings, "font_name", &original_font_name);
                //set_setting_int_to_q_setting(&q_settings, "font_size", original_font_size);

                q_settings.sync();

                init_settings(&main_window);
                if let Err(error) = ui.load() {
                    return show_dialog(&ui.dialog, error, false);
                }

                // Once the original settings are reloaded, wipe them out from the backend again and put the old ones in.
                // That way, if the user cancels, we still have the old settings.
                q_settings.clear();
                q_settings.sync();

                for (key, value) in &old_settings {
                    set_setting_variant_to_q_setting(&q_settings, key, value.as_ref());
                }

                // Set this value to indicate future operations that a reset has taken place.
                set_setting_bool_to_q_setting(&q_settings, "factoryReset", true);

                // Save the backend settings again.
                q_settings.sync();
            }
        ));

        // What happens when we hit any of the "..." buttons for the games.
        let mut select_game_paths = BTreeMap::new();
        for key in ui.paths_games_line_edits.keys() {
            select_game_paths.insert(
                key.to_owned(),
                SlotNoArgs::new(&ui.dialog, clone!(
                    key,
                    ui => move || {
                    ui.update_entry_path(&key);
                }))
            );
        }

        Self {
            restore_default,
            select_game_paths,
        }
    }
}
