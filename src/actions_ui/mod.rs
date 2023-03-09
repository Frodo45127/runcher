//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QGridLayout;
use qt_widgets::QMainWindow;
use qt_widgets::QToolButton;

use qt_core::QBox;
use qt_core::QPtr;

use anyhow::Result;
use getset::*;

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

        layout.add_widget_5a(&main_widget, 0, 1, 1, 1);

        Ok(Self {
            play_button,
            settings_button,
        })
    }
}
