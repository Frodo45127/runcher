//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QGridLayout;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QToolButton;

use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;

use anyhow::Result;
use getset::*;

use std::collections::BTreeMap;
use std::fs::DirBuilder;

use rpfm_lib::games::supported_games::KEY_ARENA;

use rpfm_ui_common::locale::*;
use rpfm_ui_common::QUALIFIER;
use rpfm_ui_common::ORGANISATION;
use rpfm_ui_common::PROGRAM_NAME;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::SUPPORTED_GAMES;

const VIEW_DEBUG: &str = "ui_templates/settings_dialog.ui";
const VIEW_RELEASE: &str = "ui/settings_dialog.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//


#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct SettingsUI {
    paths_games_line_edits: BTreeMap<String, QBox<QLineEdit>>,
    paths_games_buttons: BTreeMap<String, QBox<QToolButton>>,

    default_game_model: QBox<QStandardItemModel>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl SettingsUI {

    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Self> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

        let paths_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "paths_groupbox")?;
        let default_game_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "default_game_label")?;
        let default_game_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "default_game_combobox")?;
        let paths_layout: QPtr<QGridLayout> = paths_groupbox.layout().static_downcast();
        let default_game_model = QStandardItemModel::new_1a(&default_game_combobox);
        default_game_combobox.set_model(&default_game_model);

        // We automatically add a Label/LineEdit/Button for each game we support.
        let mut paths_games_line_edits = BTreeMap::new();
        let mut paths_games_buttons = BTreeMap::new();

        for (index, game) in SUPPORTED_GAMES.games_sorted().iter().enumerate() {
            if game.game_key_name() != KEY_ARENA {
                let game_key = game.game_key_name();
                let game_label = QLabel::from_q_string_q_widget(&QString::from_std_str(game.display_name()), &paths_groupbox);
                let game_line_edit = QLineEdit::from_q_widget(&paths_groupbox);
                let game_button = QToolButton::new_1a(&paths_groupbox);
                game_line_edit.set_placeholder_text(&qtre("settings_game_line_ph", &[game.display_name()]));

                paths_layout.add_widget_5a(&game_label, index as i32, 0, 1, 1);
                paths_layout.add_widget_5a(&game_line_edit, index as i32, 1, 1, 1);
                paths_layout.add_widget_5a(&game_button, index as i32, 2, 1, 1);

                // Add the LineEdit and Button to the list.
                paths_games_line_edits.insert(game_key.to_owned(), game_line_edit);
                paths_games_buttons.insert(game_key.to_owned(), game_button);

                // Add the game to the default game combo.
                default_game_combobox.add_item_q_string(&QString::from_std_str(game.display_name()));
            }
        }

        main_widget.dynamic_cast::<QDialog>().exec();
        //settings_ui.load()?;
        //if settings_ui.dialog.exec() == 1 {
        //    settings_ui.save()?;
        //    settings_ui.dialog.delete_later();
        //    Ok(true)
        //} else {
        //    Ok(false)
        //}

        Ok(Self {
            paths_games_line_edits,
            paths_games_buttons,
            default_game_model,
        })
    }
}

//-------------------------------------------------------------------------------//
//                         Setting-related functions
//-------------------------------------------------------------------------------//

pub unsafe fn init_settings(main_window: &QPtr<QMainWindow>) {
    let q_settings = settings();

    set_setting_if_new_q_byte_array(&q_settings, "originalGeometry", main_window.save_geometry().as_ref());
    set_setting_if_new_q_byte_array(&q_settings, "originalWindowState", main_window.save_state_0a().as_ref());

    q_settings.sync();
}

//-------------------------------------------------------------------------------//
//                             Extra Helpers
//-------------------------------------------------------------------------------//

#[must_use = "Many things depend on this folder existing. So better check this worked."]
pub fn init_config_path() -> Result<()> {

    *QUALIFIER.write().unwrap() = "com".to_owned();
    *ORGANISATION.write().unwrap() = "FrodoWazEre".to_owned();
    *PROGRAM_NAME.write().unwrap() = "runcher".to_owned();

    DirBuilder::new().recursive(true).create(error_path()?)?;

    Ok(())
}
