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
use qt_widgets::QGridLayout;
use qt_widgets::QMainWindow;
use qt_widgets::QToolButton;

use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;

use anyhow::Result;
use getset::*;

use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::utils::*;

const VIEW_DEBUG: &str = "ui_templates/actions_groupbox.ui";
const VIEW_RELEASE: &str = "ui/actions_groupbox.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//


#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ActionsUI {
    play_button: QPtr<QToolButton>,
    settings_button: QPtr<QToolButton>,

    profile_load_button: QPtr<QToolButton>,
    profile_save_button: QPtr<QToolButton>,
    profile_combobox: QPtr<QComboBox>,
    profile_model: QBox<QStandardItemModel>,

}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ActionsUI {

    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Self> {
        let layout: QPtr<QGridLayout> = main_window.central_widget().layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

        let play_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "play_button")?;
        let settings_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "settings_button")?;
        play_button.set_tool_tip(&qtr("launch_game"));
        settings_button.set_tool_tip(&qtr("settings"));

        let profile_load_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "profile_load_button")?;
        let profile_save_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "profile_save_button")?;
        let profile_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "profile_combobox")?;
        let profile_model: QBox<QStandardItemModel> = QStandardItemModel::new_1a(&profile_combobox);
        profile_combobox.set_model(&profile_model);
        profile_load_button.set_tool_tip(&qtr("load_profile"));
        profile_save_button.set_tool_tip(&qtr("save_profile"));

        layout.add_widget_5a(&main_widget, 0, 1, 1, 1);

        Ok(Self {
            play_button,
            settings_button,

            profile_load_button,
            profile_save_button,
            profile_combobox,
            profile_model,
        })
    }
}
