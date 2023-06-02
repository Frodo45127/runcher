//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QAction;
use qt_widgets::QComboBox;
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QMenu;
use qt_widgets::{QToolButton, q_tool_button::ToolButtonPopupMode};
use qt_widgets::QWidget;
use qt_widgets::QWidgetAction;

use qt_gui::QIcon;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;

use anyhow::Result;
use getset::*;

use std::sync::Arc;

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
    enable_logging: QPtr<QAction>,
    enable_skip_intro: QPtr<QAction>,
    _unit_multiplier: QBox<QWidgetAction>,
    unit_multiplier_spinbox: QBox<QDoubleSpinBox>,

    settings_button: QPtr<QToolButton>,
    folders_button: QPtr<QToolButton>,
    open_game_root_folder: QPtr<QAction>,
    open_game_data_folder: QPtr<QAction>,
    open_game_content_folder: QPtr<QAction>,
    open_runcher_config_folder: QPtr<QAction>,
    open_runcher_error_folder: QPtr<QAction>,

    copy_load_order_button: QPtr<QToolButton>,
    paste_load_order_button: QPtr<QToolButton>,
    reload_button: QPtr<QToolButton>,

    profile_load_button: QPtr<QToolButton>,
    profile_save_button: QPtr<QToolButton>,
    profile_combobox: QPtr<QComboBox>,
    profile_model: QBox<QStandardItemModel>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ActionsUI {

    pub unsafe fn new(parent: &QBox<QWidget>) -> Result<Arc<Self>> {
        let layout: QPtr<QGridLayout> = parent.layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        let play_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "play_button")?;
        let play_menu = QMenu::from_q_widget(&play_button);
        let enable_logging = play_menu.add_action_q_string(&qtr("enable_logging"));
        let enable_skip_intro = play_menu.add_action_q_string(&qtr("enable_skip_intro"));
        enable_logging.set_checkable(true);
        enable_skip_intro.set_checkable(true);
        play_button.set_popup_mode(ToolButtonPopupMode::MenuButtonPopup);

        let unit_multiplier = QWidgetAction::new(&play_menu);
        let unit_multiplier_widget = QWidget::new_1a(&play_menu);
        let unit_multiplier_label = QLabel::from_q_string_q_widget(&qtr("unit_multiplier"), &unit_multiplier_widget);
        let unit_multiplier_spinbox = QDoubleSpinBox::new_1a(&unit_multiplier_widget);
        let unit_multiplier_layout = create_grid_layout(unit_multiplier_widget.static_upcast());
        unit_multiplier_layout.add_widget_5a(&unit_multiplier_label, 0, 0, 1, 1);
        unit_multiplier_layout.add_widget_5a(&unit_multiplier_spinbox, 0, 1, 1, 1);
        unit_multiplier_spinbox.set_value(1.00);
        unit_multiplier.set_default_widget(&unit_multiplier_widget);
        play_menu.add_action(&unit_multiplier);
        play_button.set_menu(play_menu.into_raw_ptr());

        let settings_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "settings_button")?;
        let folders_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "folders_button")?;
        play_button.set_tool_tip(&qtr("launch_game"));
        settings_button.set_tool_tip(&qtr("settings"));
        folders_button.set_tool_tip(&qtr("open_folders"));

        let folders_menu = QMenu::from_q_widget(&folders_button);
        let open_game_root_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_game_root_folder"));
        let open_game_data_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_game_data_folder"));
        let open_game_content_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_game_content_folder"));
        let open_runcher_config_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_runcher_config_folder"));
        let open_runcher_error_folder = folders_menu.add_action_q_icon_q_string(&QIcon::from_theme_1a(&QString::from_std_str("folder")), &qtr("open_runcher_error_folder"));
        folders_button.set_menu(folders_menu.into_raw_ptr());
        folders_button.set_popup_mode(ToolButtonPopupMode::MenuButtonPopup);

        let copy_load_order_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "copy_load_order_button")?;
        let paste_load_order_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "paste_load_order_button")?;
        let reload_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "reload_button")?;
        copy_load_order_button.set_tool_tip(&qtr("copy_load_order"));
        paste_load_order_button.set_tool_tip(&qtr("paste_load_order"));
        reload_button.set_tool_tip(&qtr("reload"));

        let profile_load_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "profile_load_button")?;
        let profile_save_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "profile_save_button")?;
        let profile_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "profile_combobox")?;
        let profile_model: QBox<QStandardItemModel> = QStandardItemModel::new_1a(&profile_combobox);
        profile_combobox.set_model(&profile_model);
        profile_combobox.line_edit().set_placeholder_text(&qtr("profile_name"));
        profile_load_button.set_tool_tip(&qtr("load_profile"));
        profile_save_button.set_tool_tip(&qtr("save_profile"));

        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        let ui = Arc::new(Self {
            play_button,
            enable_logging,
            enable_skip_intro,
            _unit_multiplier: unit_multiplier,
            unit_multiplier_spinbox,

            settings_button,
            folders_button,
            open_game_root_folder,
            open_game_data_folder,
            open_game_content_folder,
            open_runcher_config_folder,
            open_runcher_error_folder,

            copy_load_order_button,
            paste_load_order_button,
            reload_button,

            profile_load_button,
            profile_save_button,
            profile_combobox,
            profile_model,
        });

        Ok(ui)
    }
}
