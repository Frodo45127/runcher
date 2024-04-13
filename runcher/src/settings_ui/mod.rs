//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QAction;
use qt_widgets::QApplication;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::q_dialog_button_box::{ButtonRole, StandardButton};
use qt_widgets::{QFrame, q_frame::Shape};
use qt_widgets::{QFileDialog, q_file_dialog::{FileMode, Option as QFileDialogOption}};
use qt_widgets::QGridLayout;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QMenu;
use qt_widgets::QPushButton;
use qt_widgets::QTableView;
use qt_widgets::QToolButton;

use qt_gui::QIcon;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QCoreApplication;
use qt_core::QFlags;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QString;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use getset::*;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::{DirBuilder, File};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::UNIX_EPOCH;

use rpfm_lib::games::{GameInfo, supported_games::{KEY_ARENA, KEY_SHOGUN_2, KEY_WARHAMMER_3}};

use rpfm_ui_common::locale::*;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::ffi::*;
use crate::mod_manager::tools::{Tools, Tool};
use crate::SUPPORTED_GAMES;
use crate::updater_ui::*;

use self::slots::SettingsUISlots;

mod slots;

const VIEW_DEBUG: &str = "ui_templates/settings_dialog.ui";
const VIEW_RELEASE: &str = "ui/settings_dialog.ui";

pub const SLASH_DMY_DATE_FORMAT_STR: &str = "[day]/[month]/[year]";
pub const SLASH_MDY_DATE_FORMAT_STR: &str = "[month]/[day]/[year]";
pub const SLASH_YMD_DATE_FORMAT_STR: &str = "[year]/[month]/[day]";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct SettingsUI {
    dialog: QPtr<QDialog>,

    font_data: Rc<RefCell<(String, i32)>>,

    paths_games_line_edits: BTreeMap<String, QBox<QLineEdit>>,
    paths_games_buttons: BTreeMap<String, QBox<QToolButton>>,

    secondary_mods_folder_line_edit: QBox<QLineEdit>,
    secondary_mods_folder_button: QBox<QToolButton>,

    tools_tableview: QPtr<QTableView>,
    tools_model: QBox<QStandardItemModel>,
    tools_context_menu: QBox<QMenu>,
    tools_add: QPtr<QAction>,
    tools_remove: QPtr<QAction>,

    steam_api_key_line_edit: QPtr<QLineEdit>,

    language_combobox: QPtr<QComboBox>,
    default_game_combobox: QPtr<QComboBox>,
    update_chanel_combobox: QPtr<QComboBox>,
    date_format_combobox: QPtr<QComboBox>,
    check_updates_on_start_checkbox: QPtr<QCheckBox>,
    check_schema_updates_on_start_checkbox: QPtr<QCheckBox>,
    dark_mode_checkbox: QPtr<QCheckBox>,
    open_workshop_link_in_steam_checkbox: QPtr<QCheckBox>,
    check_logs_checkbox: QPtr<QCheckBox>,
    enable_debug_terminal_checkbox: QPtr<QCheckBox>,

    font_button: QBox<QPushButton>,
    restore_default_button: QPtr<QPushButton>,
    accept_button: QPtr<QPushButton>,
    cancel_button: QPtr<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl SettingsUI {

    /// This function creates a ***Settings*** dialog, execute it, and returns a new `Settings`, or `None` if you close/cancel the dialog.
    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<bool> {
        let settings_ui = Self::new_with_parent(main_window)?;
        let slots = SettingsUISlots::new(&settings_ui, main_window.static_upcast());
        settings_ui.set_connections(&slots);

        // If load fails due to missing locale folder, show the error and cancel the settings edition.
        settings_ui.load()?;

        if settings_ui.dialog.exec() == 1 {
            settings_ui.save()?;
            settings_ui.dialog.delete_later();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub unsafe fn new_with_parent(main_window: &QBox<QMainWindow>) -> Result<Rc<Self>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;
        let dialog: QPtr<QDialog> = main_widget.static_downcast();

        let tools_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "tools_groupbox")?;
        let tools_tableview: QPtr<QTableView> = find_widget(&main_widget.static_upcast(), "tools_tableview")?;
        let tools_model = QStandardItemModel::new_1a(&tools_tableview);
        tools_tableview.set_model(&tools_model);
        tools_groupbox.set_title(&qtr("tools_title"));
        path_item_delegate_safe(&tools_tableview.static_upcast::<QObject>().as_ptr(), 1);
        game_selector_item_delegate_safe(&tools_tableview.static_upcast::<QObject>().as_ptr(), 2);

        let tools_context_menu = QMenu::from_q_widget(&main_widget);
        let tools_add = tools_context_menu.add_action_q_string(&qtr("tools_add"));
        let tools_remove = tools_context_menu.add_action_q_string(&qtr("tools_remove"));

        let paths_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "paths_groupbox")?;
        let language_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "language_label")?;
        let default_game_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "default_game_label")?;
        let update_chanel_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "update_chanel_label")?;
        let date_format_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "date_format_label")?;
        let steam_api_key_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "steam_api_key_label")?;
        let check_updates_on_start_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "check_updates_on_start_label")?;
        let check_schema_updates_on_start_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "check_schema_updates_on_start_label")?;
        let dark_mode_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "dark_mode_label")?;
        let open_workshop_link_in_steam_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "open_workshop_link_in_steam_label")?;
        let check_logs_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "check_logs_label")?;
        let enable_debug_terminal_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "enable_debug_terminal_label")?;
        let language_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "language_combobox")?;
        let default_game_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "default_game_combobox")?;
        let update_chanel_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "update_chanel_combobox")?;
        let date_format_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "date_format_combobox")?;
        let steam_api_key_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "steam_api_key_line_edit")?;
        let check_updates_on_start_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "check_updates_on_start_checkbox")?;
        let check_schema_updates_on_start_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "check_schema_updates_on_start_checkbox")?;
        let dark_mode_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "dark_mode_checkbox")?;
        let open_workshop_link_in_steam_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "open_workshop_link_in_steam_checkbox")?;
        let check_logs_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "check_logs_checkbox")?;
        let enable_debug_terminal_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "enable_debug_terminal_checkbox")?;
        let paths_layout: QPtr<QGridLayout> = paths_groupbox.layout().static_downcast();
        update_chanel_combobox.add_item_q_string(&QString::from_std_str(STABLE));
        update_chanel_combobox.add_item_q_string(&QString::from_std_str(BETA));
        date_format_combobox.add_item_q_string(&QString::from_std_str(SLASH_DMY_DATE_FORMAT_STR));
        date_format_combobox.add_item_q_string(&QString::from_std_str(SLASH_MDY_DATE_FORMAT_STR));
        date_format_combobox.add_item_q_string(&QString::from_std_str(SLASH_YMD_DATE_FORMAT_STR));

        paths_groupbox.set_title(&qtr("game_paths"));
        language_label.set_text(&qtr("language"));
        default_game_label.set_text(&qtr("default_game"));
        update_chanel_label.set_text(&qtr("update_channel"));
        date_format_label.set_text(&qtr("date_format"));
        steam_api_key_label.set_text(&qtr("steam_api_key"));
        check_updates_on_start_label.set_text(&qtr("check_updates_on_start"));
        check_schema_updates_on_start_label.set_text(&qtr("check_schema_updates_on_start"));
        dark_mode_label.set_text(&qtr("dark_mode"));
        open_workshop_link_in_steam_label.set_text(&qtr("open_workshop_link_in_steam"));
        check_logs_label.set_text(&qtr("check_logs"));
        enable_debug_terminal_label.set_text(&qtr("enable_debug_terminal"));

        // Add one path at the beginning for the secondary mods folder.
        let secondary_mods_folder_label = QLabel::from_q_string_q_widget(&qtr("settings_secondary_mods_folder"), &paths_groupbox);
        let secondary_mods_folder_line_edit = QLineEdit::from_q_widget(&paths_groupbox);
        let secondary_mods_folder_button = QToolButton::new_1a(&paths_groupbox);
        secondary_mods_folder_line_edit.set_placeholder_text(&qtr("settings_secondary_mods_folder_ph"));
        secondary_mods_folder_button.set_icon(&QIcon::from_theme_1a(&QString::from_std_str("folder")));

        paths_layout.add_widget_5a(&secondary_mods_folder_label, 0, 0, 1, 1);
        paths_layout.add_widget_5a(&secondary_mods_folder_line_edit, 0, 1, 1, 1);
        paths_layout.add_widget_5a(&secondary_mods_folder_button, 0, 2, 1, 1);

        // TODO: Maybe add a separator here.
        let line = QFrame::new_1a(&paths_groupbox);
        line.set_frame_shape(Shape::HLine);
        paths_layout.add_widget_5a(&line, 1, 0, 1, 3);

        // We automatically add a Label/LineEdit/Button for each game we support.
        let mut paths_games_line_edits = BTreeMap::new();
        let mut paths_games_buttons = BTreeMap::new();

        for (index, game) in SUPPORTED_GAMES.games_sorted().iter().enumerate() {
            if game.key() != KEY_ARENA {
                let game_key = game.key();
                let game_label = QLabel::from_q_string_q_widget(&QString::from_std_str(game.display_name()), &paths_groupbox);
                let game_line_edit = QLineEdit::from_q_widget(&paths_groupbox);
                let game_button = QToolButton::new_1a(&paths_groupbox);
                game_line_edit.set_placeholder_text(&qtre("settings_game_line_ph", &[game.display_name()]));
                game_button.set_icon(&QIcon::from_theme_1a(&QString::from_std_str("folder")));

                paths_layout.add_widget_5a(&game_label, index as i32 + 2, 0, 1, 1);
                paths_layout.add_widget_5a(&game_line_edit, index as i32 + 2, 1, 1, 1);
                paths_layout.add_widget_5a(&game_button, index as i32 + 2, 2, 1, 1);

                // Add the LineEdit and Button to the list.
                paths_games_line_edits.insert(game_key.to_owned(), game_line_edit);
                paths_games_buttons.insert(game_key.to_owned(), game_button);

                // Add the game to the default game combo.
                default_game_combobox.add_item_q_string(&QString::from_std_str(game.display_name()));
            }
        }

        if let Ok(locales) = Locale::get_available_locales() {
            for (language, _) in locales {
                language_combobox.add_item_q_string(&QString::from_std_str(language));
            }
        }

        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        let font_button = QPushButton::from_q_string_q_widget(&qtr("settings_font_title"), &button_box);
        button_box.add_button_q_abstract_button_button_role(&font_button, ButtonRole::ResetRole);

        let restore_default_button: QPtr<QPushButton> = button_box.button(StandardButton::RestoreDefaults);
        let accept_button: QPtr<QPushButton> = button_box.button(StandardButton::Ok);
        let cancel_button: QPtr<QPushButton> = button_box.button(StandardButton::Cancel);

        Ok(Rc::new(Self {
            dialog,
            font_data: Rc::new(RefCell::new((String::new(), -1))),

            tools_tableview,
            tools_model,
            tools_context_menu,
            tools_add,
            tools_remove,

            paths_games_line_edits,
            paths_games_buttons,

            secondary_mods_folder_line_edit,
            secondary_mods_folder_button,

            steam_api_key_line_edit,
            language_combobox,
            default_game_combobox,
            update_chanel_combobox,
            date_format_combobox,
            check_updates_on_start_checkbox,
            check_schema_updates_on_start_checkbox,
            dark_mode_checkbox,
            open_workshop_link_in_steam_checkbox,
            check_logs_checkbox,
            enable_debug_terminal_checkbox,

            font_button,
            restore_default_button,
            accept_button,
            cancel_button,
        }))
    }


    /// This function loads the data from the provided `Settings` into our `SettingsUI`.
    pub unsafe fn load(&self) -> Result<()> {

        // Tools are kept in a json, not in a qsetting, for ease of updating. Load it first.
        let tools = Tools::load().unwrap_or_else(|_| Tools::default());
        self.tools_model().clear();

        // Build the columns.
        self.tools_model().set_column_count(3);
        self.tools_model().set_horizontal_header_item(0, QStandardItem::from_q_string(&qtr("tools_column_name")).into_ptr());
        self.tools_model().set_horizontal_header_item(1, QStandardItem::from_q_string(&qtr("tools_column_path")).into_ptr());
        self.tools_model().set_horizontal_header_item(2, QStandardItem::from_q_string(&qtr("tools_column_games")).into_ptr());

        for tool in tools.tools() {
            let row = QListOfQStandardItem::new();

            let item_name = QStandardItem::new();
            let item_path = QStandardItem::new();
            let item_games = QStandardItem::new();

            item_name.set_text(&QString::from_std_str(tool.name()));
            item_path.set_text(&QString::from_std_str(tool.path().to_string_lossy()));
            item_games.set_text(&QString::from_std_str(tool.games().join(",")));

            row.append_q_standard_item(&item_name.into_ptr().as_mut_raw_ptr());
            row.append_q_standard_item(&item_path.into_ptr().as_mut_raw_ptr());
            row.append_q_standard_item(&item_games.into_ptr().as_mut_raw_ptr());

            self.tools_model().append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());
        }

        self.tools_tableview().horizontal_header().resize_sections(ResizeMode::ResizeToContents);

        let q_settings = settings();
        let secondary_mods_path = setting_string_from_q_setting(&q_settings, "secondary_mods_path");
        if !secondary_mods_path.is_empty() {
            self.secondary_mods_folder_line_edit().set_text(&QString::from_std_str(secondary_mods_path));
        }

        // Load the Game Paths, if they exists.
        for (key, path) in self.paths_games_line_edits.iter() {
            let stored_path = setting_string_from_q_setting(&q_settings, key);
            if !stored_path.is_empty() {
                path.set_text(&QString::from_std_str(stored_path));
            }
        }

        // Get the default game.
        let default_game = setting_string_from_q_setting(&q_settings, "default_game");
        for (index, game) in SUPPORTED_GAMES.games_sorted().iter().enumerate() {
            if game.key() == default_game {
                self.default_game_combobox.set_current_index(index as i32);
                break;
            }
        }

        let date_format = setting_string_from_q_setting(&q_settings, "date_format");
        for (index, format) in [SLASH_DMY_DATE_FORMAT_STR, SLASH_MDY_DATE_FORMAT_STR, SLASH_YMD_DATE_FORMAT_STR].iter().enumerate() {
            if format == &date_format {
                self.date_format_combobox.set_current_index(index as i32);
                break;
            }
        }

        let language_selected = setting_string("language");
        let language_selected_split = language_selected.split('_').collect::<Vec<&str>>()[0];
        for (index, (language,_)) in Locale::get_available_locales()?.iter().enumerate() {
            if *language == language_selected_split {
                self.language_combobox.set_current_index(index as i32);
                break;
            }
        }

        for (index, update_channel_name) in [UpdateChannel::Stable, UpdateChannel::Beta].iter().enumerate() {
            if update_channel_name == &update_channel() {
                self.update_chanel_combobox.set_current_index(index as i32);
                break;
            }
        }

        *self.font_data.borrow_mut() = (setting_string_from_q_setting(&q_settings, "font_name"), setting_int_from_q_setting(&q_settings, "font_size"));

        self.steam_api_key_line_edit().set_text(&QString::from_std_str(setting_string_from_q_setting(&q_settings, "steam_api_key")));
        self.dark_mode_checkbox().set_checked(setting_bool_from_q_setting(&q_settings, "dark_mode"));
        self.open_workshop_link_in_steam_checkbox().set_checked(setting_bool_from_q_setting(&q_settings, "open_workshop_link_in_steam"));
        self.check_updates_on_start_checkbox().set_checked(setting_bool_from_q_setting(&q_settings, "check_updates_on_start"));
        self.check_schema_updates_on_start_checkbox().set_checked(setting_bool_from_q_setting(&q_settings, "check_schema_updates_on_start"));
        self.check_logs_checkbox().set_checked(setting_bool_from_q_setting(&q_settings, "check_logs"));
        self.enable_debug_terminal_checkbox().set_checked(setting_bool_from_q_setting(&q_settings, "enable_debug_terminal"));

        Ok(())
    }

    pub unsafe fn save(&self) -> Result<()> {

        let mut tools = Tools::default();
        for row in 0..self.tools_model().row_count_0a() {
            let item_name = self.tools_model().item_2a(row, 0);
            let item_path = self.tools_model().item_2a(row, 1);
            let item_games = self.tools_model().item_2a(row, 2);

            let mut tool = Tool::default();

            *tool.name_mut() = item_name.text().to_std_string();
            *tool.path_mut() = PathBuf::from(item_path.text().to_std_string());
            *tool.games_mut() = item_games.text().to_std_string().split(',').map(|x| x.to_string()).collect::<Vec<String>>();

            tools.tools_mut().push(tool);
        }

        tools.save()?;

        // For each entry, we check if it's a valid directory and save it into Settings.
        let q_settings = settings();
        set_setting_string_to_q_setting(&q_settings, "secondary_mods_path", &self.secondary_mods_folder_line_edit().text().to_std_string());

        for (key, line_edit) in self.paths_games_line_edits.iter() {
            set_setting_string_to_q_setting(&q_settings, key, &line_edit.text().to_std_string());
        }

        // We get his game's folder, depending on the selected game.
        let mut game = self.default_game_combobox.current_text().to_std_string();
        if let Some(index) = game.find('&') { game.remove(index); }
        game = game.replace(' ', "_").to_lowercase();
        set_setting_string_to_q_setting(&q_settings, "default_game", &game);

        // We need to store the full locale filename, not just the visible name!
        let mut language = self.language_combobox.current_text().to_std_string();
        if let Some(index) = language.find('&') { language.remove(index); }
        if let Some((_, locale)) = Locale::get_available_locales()?.iter().find(|(x, _)| &language == x) {
            let file_name = format!("{}_{}", language, locale.language);
            set_setting_string_to_q_setting(&q_settings, "language", &file_name);
        }

        set_setting_string_to_q_setting(&q_settings, "font_name", &self.font_data.borrow().0);
        set_setting_int_to_q_setting(&q_settings, "font_size", self.font_data.borrow().1);

        set_setting_string_to_q_setting(&q_settings, "update_channel", &self.update_chanel_combobox.current_text().to_std_string());
        set_setting_string_to_q_setting(&q_settings, "date_format", &self.date_format_combobox.current_text().to_std_string());
        set_setting_string_to_q_setting(&q_settings, "steam_api_key", &self.steam_api_key_line_edit().text().to_std_string());
        set_setting_bool_to_q_setting(&q_settings, "dark_mode", self.dark_mode_checkbox().is_checked());
        set_setting_bool_to_q_setting(&q_settings, "open_workshop_link_in_steam", self.open_workshop_link_in_steam_checkbox().is_checked());
        set_setting_bool_to_q_setting(&q_settings, "check_updates_on_start", self.check_updates_on_start_checkbox().is_checked());
        set_setting_bool_to_q_setting(&q_settings, "check_schema_updates_on_start", self.check_schema_updates_on_start_checkbox().is_checked());
        set_setting_bool_to_q_setting(&q_settings, "check_logs", self.check_logs_checkbox().is_checked());
        set_setting_bool_to_q_setting(&q_settings, "enable_debug_terminal", self.enable_debug_terminal_checkbox().is_checked());

        // Save the settings.
        q_settings.sync();

        Ok(())
    }

    pub unsafe fn set_connections(&self, slots: &SettingsUISlots) {
        self.secondary_mods_folder_button().released().connect(slots.select_secondary_mods_path());
        for (key, button) in self.paths_games_buttons.iter() {
            button.released().connect(&slots.select_game_paths()[key]);
        }

        self.tools_tableview().custom_context_menu_requested().connect(slots.tools_context_menu());
        self.tools_tableview().selection_model().selection_changed().connect(slots.tools_enabler());
        self.tools_context_menu().about_to_show().connect(slots.tools_enabler());

        self.tools_add.triggered().connect(slots.tools_add());
        self.tools_remove.triggered().connect(slots.tools_remove());

        self.font_button.released().connect(slots.font_settings());
        self.restore_default_button.released().connect(slots.restore_default());
        self.accept_button.released().connect(self.dialog.slot_accept());
        self.cancel_button.released().connect(self.dialog.slot_close());
    }

    unsafe fn update_entry_path(&self, game: &str) {

        // We check if we have a game or not. If we have it, update the `LineEdit` for that game.
        // If we don't, update the `LineEdit` for `MyMod`s path.
        let line_edit = match self.paths_games_line_edits.get(game) {
            Some(line_edit) => line_edit,
            None => return,
        };

        // Create the `FileDialog` and configure it.
        let title = qtr("settings_select_folder");
        let file_dialog = QFileDialog::from_q_widget_q_string(
            &self.dialog,
            &title,
        );

        file_dialog.set_file_mode(FileMode::Directory);
        file_dialog.set_options(QFlags::from(QFileDialogOption::ShowDirsOnly));

        // Get the old Path, if exists.
        let old_path = line_edit.text().to_std_string();

        // If said path is not empty, and is a dir, set it as the initial directory.
        if !old_path.is_empty() && Path::new(&old_path).is_dir() {
            file_dialog.set_directory_q_string(&line_edit.text());
        }

        // Run it and expect a response (1 => Accept, 0 => Cancel).
        if file_dialog.exec() == 1 {

            // Get the path of the selected file.
            let selected_files = file_dialog.selected_files();
            let path = selected_files.at(0);

            // Add the Path to the LineEdit.
            line_edit.set_text(path);
        }
    }

    unsafe fn update_secondary_mods_path(&self) {
        let line_edit = self.secondary_mods_folder_line_edit();

        // Create the `FileDialog` and configure it.
        let title = qtr("settings_select_folder");
        let file_dialog = QFileDialog::from_q_widget_q_string(
            &self.dialog,
            &title,
        );

        file_dialog.set_file_mode(FileMode::Directory);
        file_dialog.set_options(QFlags::from(QFileDialogOption::ShowDirsOnly));

        // Get the old Path, if exists.
        let old_path = line_edit.text().to_std_string();

        // If said path is not empty, and is a dir, set it as the initial directory.
        if !old_path.is_empty() && Path::new(&old_path).is_dir() {
            file_dialog.set_directory_q_string(&line_edit.text());
        }

        // Run it and expect a response (1 => Accept, 0 => Cancel).
        if file_dialog.exec() == 1 {

            // Get the path of the selected file.
            let selected_files = file_dialog.selected_files();
            let path = selected_files.at(0);

            // Add the Path to the LineEdit.
            line_edit.set_text(path);
        }
    }
}

//-------------------------------------------------------------------------------//
//                         Setting-related functions
//-------------------------------------------------------------------------------//

pub unsafe fn init_settings(main_window: &QPtr<QMainWindow>) {
    let q_settings = settings();

    set_setting_if_new_q_byte_array(&q_settings, "originalGeometry", main_window.save_geometry().as_ref());
    set_setting_if_new_q_byte_array(&q_settings, "originalWindowState", main_window.save_state_0a().as_ref());

    let font = QApplication::font();
    let font_name = font.family().to_std_string();
    let font_size = font.point_size();
    set_setting_if_new_string(&q_settings, "font_name", &font_name);
    set_setting_if_new_int(&q_settings, "font_size", font_size);
    set_setting_if_new_string(&q_settings, "original_font_name", &font_name);
    set_setting_if_new_int(&q_settings, "original_font_size", font_size);

    set_setting_if_new_string(&q_settings, "steam_api_key", "");
    set_setting_if_new_string(&q_settings, "default_game", KEY_WARHAMMER_3);
    set_setting_if_new_string(&q_settings, "update_channel", "stable");
    set_setting_if_new_string(&q_settings, "language", "English_en");
    set_setting_if_new_string(&q_settings, "date_format", SLASH_DMY_DATE_FORMAT_STR);
    set_setting_if_new_bool(&q_settings, "check_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "check_schema_updates_on_start", true);
    set_setting_if_new_bool(&q_settings, "dark_mode", false);
    set_setting_if_new_bool(&q_settings, "check_logs", true);
    set_setting_if_new_bool(&q_settings, "enable_debug_terminal", false);

    for game in &SUPPORTED_GAMES.games_sorted() {
        if game.key() != KEY_ARENA && game.key() != KEY_SHOGUN_2 {
            set_setting_if_new_bool(&q_settings, &format!("enable_logging_{}", game.key()), false);
            set_setting_if_new_bool(&q_settings, &format!("enable_skip_intros_{}", game.key()), false);
            set_setting_if_new_string(&q_settings, &format!("enable_translations_{}", game.key()), "--");
            set_setting_if_new_bool(&q_settings, &format!("merge_all_mods_{}", game.key()), false);

            let game_path = if let Ok(Some(game_path)) = game.find_game_install_location() {
                game_path.to_string_lossy().to_string()
            } else {
                String::new()
            };

            // If we got a path and we don't have it saved yet, save it automatically.
            let current_path = setting_string_from_q_setting(&q_settings, game.key());
            if current_path.is_empty() && !game_path.is_empty() {
                set_setting_string_to_q_setting(&q_settings, game.key(), &game_path);
            } else {
                set_setting_if_new_string(&q_settings, game.key(), &game_path);
            }
        }
    }

    q_settings.sync();
}

//-------------------------------------------------------------------------------//
//                             Extra Helpers
//-------------------------------------------------------------------------------//

#[must_use = "Many things depend on this folder existing. So better check this worked."]
pub fn init_config_path() -> Result<()> {
    DirBuilder::new().recursive(true).create(error_path()?)?;
    DirBuilder::new().recursive(true).create(game_config_path()?)?;
    DirBuilder::new().recursive(true).create(profiles_path()?)?;
    DirBuilder::new().recursive(true).create(schemas_path()?)?;

    DirBuilder::new().recursive(true).create(translations_local_path()?)?;
    DirBuilder::new().recursive(true).create(translations_remote_path()?)?;

    // Within the config path we need to create a folder to store the temp packs of each game.
    // Otherwise they interfere with each other due to being movie packs.
    for game in SUPPORTED_GAMES.games_sorted().iter() {
        if game.key() != KEY_ARENA {
            DirBuilder::new().recursive(true).create(config_path()?.join("temp_packs").join(game.key()))?;
        }
    }

    Ok(())
}

pub fn temp_packs_folder(game: &GameInfo) -> Result<PathBuf> {
    Ok(config_path()?.join("temp_packs").join(game.key()))
}

pub fn schemas_path() -> Result<PathBuf> {
    Ok(config_path()?.join("schemas"))
}

pub fn game_config_path() -> Result<PathBuf> {
    Ok(config_path()?.join("game_config"))
}

pub fn profiles_path() -> Result<PathBuf> {
    Ok(config_path()?.join("profiles"))
}

pub fn rpfm_config_path() -> Result<PathBuf> {
    if cfg!(debug_assertions) { std::env::current_dir().map_err(From::from) } else {
        unsafe {
            match ProjectDirs::from(&QCoreApplication::organization_domain().to_std_string(), &QCoreApplication::organization_name().to_std_string(), "rpfm") {
                Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
                None => Err(anyhow!("Failed to get RPFM's config path."))
            }
        }
    }
}

pub fn translations_local_path() -> Result<PathBuf> {
    rpfm_config_path().map(|path| path.join("translations_local"))
}

pub fn translations_remote_path() -> Result<PathBuf> {
    config_path().map(|path| path.join("translations_remote"))
}

pub fn last_game_update_date(game: &GameInfo, game_path: &Path) -> Result<u64> {
    Ok(if let Some(exe_path) = game.executable_path(game_path) {
        if let Ok(exe) = File::open(exe_path) {
            if cfg!(target_os = "windows") {
                exe.metadata()?.created()?.duration_since(UNIX_EPOCH)?.as_secs()
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    })
}
