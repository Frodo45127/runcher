//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QCheckBox;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::{QFileDialog, q_file_dialog::{FileMode, Option as QFileDialogOption}};
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QToolButton;
use qt_widgets::QWidget;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QFlags;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::SlotNoArgs;

use cpp_core::Ref;

use anyhow::{anyhow, Result};
use getset::*;
use itertools::Itertools;
use mslnk::ShellLink;

use std::path::{PathBuf, Path};
use std::rc::Rc;

use rpfm_ui_common::clone;
use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::utils::*;

use crate::AppUI;
use crate::profiles_ui::slots::ProfilesUISlots;

const VIEW_DEBUG: &str = "ui_templates/profile_manager_dialog.ui";
const VIEW_RELEASE: &str = "ui/profile_manager_dialog.ui";

const RENAME_VIEW_DEBUG: &str = "ui_templates/profile_rename_dialog.ui";
const RENAME_VIEW_RELEASE: &str = "ui/profile_rename_dialog.ui";

const SHORTCUT_VIEW_DEBUG: &str = "ui_templates/profile_shortcut_dialog.ui";
const SHORTCUT_VIEW_RELEASE: &str = "ui/profile_shortcut_dialog.ui";

mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ProfilesUI {
    main_widget: QBox<QWidget>,
    details_label: QPtr<QLabel>,
    profiles_list_view: QPtr<QListView>,
    profiles_list_model: QBox<QStandardItemModel>,
    rename_profile_button: QPtr<QToolButton>,
    delete_profile_button: QPtr<QToolButton>,
    shortcut_button: QPtr<QToolButton>,

}

//---------------------------------------------------------------------------//
//                              UI functions
//---------------------------------------------------------------------------//

impl ProfilesUI {

    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Result<()> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;

        let details_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "details_groupbox")?;
        let details_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "details_label")?;

        let rename_profile_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "rename_button")?;
        let delete_profile_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "delete_button")?;
        let shortcut_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "shortcut_button")?;
        let profiles_list_view: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "profiles_list_view")?;
        let profiles_list_model = QStandardItemModel::new_1a(&profiles_list_view);
        profiles_list_view.set_model(&profiles_list_model);

        details_groupbox.set_title(&qtr("profile_details_title"));
        details_label.set_open_external_links(true);
        rename_profile_button.set_tool_tip(&qtr("profile_rename"));
        delete_profile_button.set_tool_tip(&qtr("profile_delete"));
        shortcut_button.set_tool_tip(&qtr("profile_shortcut_new"));

        // Disable the buttons.
        delete_profile_button.set_enabled(false);
        rename_profile_button.set_enabled(false);
        shortcut_button.set_enabled(false);

        let ui = Rc::new(Self {
            main_widget,
            details_label,
            profiles_list_view,
            profiles_list_model,
            rename_profile_button,
            delete_profile_button,
            shortcut_button,
        });

        let slots = ProfilesUISlots::new(&ui, app_ui);
        ui.set_connections(&slots);

        ui.load_data(app_ui);

        ui.dialog().set_window_title(&qtr("profile_manager_title"));
        ui.dialog().exec();

        Ok(())
    }

    pub unsafe fn set_connections(&self, slots: &ProfilesUISlots) {
        self.profiles_list_view().selection_model().selection_changed().connect(slots.update_details());

        self.rename_profile_button().released().connect(slots.profile_rename());
        self.delete_profile_button().released().connect(slots.profile_delete());
        self.shortcut_button().released().connect(slots.profile_shorcut());
    }

    pub unsafe fn load_data(&self, app_ui: &Rc<AppUI>) {
        let profiles = app_ui.game_profiles().read().unwrap();
        profiles.values()
            .sorted_by_key(|profile| profile.id())
            .for_each(|profile| {
                let item = QStandardItem::new();
                item.set_text(&QString::from_std_str(profile.id()));
                self.profiles_list_model().append_row_q_standard_item(item.into_ptr());
            });
    }

    pub unsafe fn dialog(&self) -> QPtr<QDialog> {
        self.main_widget().static_downcast::<QDialog>()
    }

    pub unsafe fn load_entry_to_detailed_view(&self, app_ui: &Rc<AppUI>, index: Ref<QModelIndex>) {
        let mut details = String::new();
        details.push_str("<ul>");

        let profile_id = index.data_0a().to_string().to_std_string();
        let profiles = app_ui.game_profiles().read().unwrap();
        if let Some(profile) = profiles.get(&profile_id) {
            details.push_str(&format!("<li>Profile ID/Name: {}</li>", profile.id()));
            details.push_str(&format!("<li>Game: {}</li>", profile.game()));

            if profile.load_order().mods().is_empty() {
                details.push_str("<li>Profile contains an empty load order.</li>");
            } else if let Some(ref game_config) = *app_ui.game_config().read().unwrap() {
                let mods = profile.load_order().mods()
                    .iter()
                    .sorted()
                    .map(|mod_id| (mod_id, game_config.mods().get(mod_id)))
                    .collect::<Vec<_>>();

                details.push_str("<li>This profile contains the following load order:</li><ul>");
                details.push_str(&format!("<li>Mode: {}</li>", if *profile.load_order().automatic() { "Automatic" } else { "Manual" }));
                details.push_str("<li>Order:</li><ul>");

                for (mod_id, modd) in &mods {
                    let link = match modd {
                        Some(modd) => match modd.steam_id() {
                            Some(steam_id) => format!("<a href=\"https://steamcommunity.com/sharedfiles/filedetails/?id={}\">(Download Link)</a>", steam_id),
                            None => String::new(),
                        },
                        None => String::new(),
                    };

                    details.push_str(&format!("<li>{}<b>{}</b> <i>({})</i></li>", link, mod_id, match modd {
                        Some(modd) => modd.name(),
                        None => "Not Installed",
                    }));
                }

                details.push_str("</ul></ul>");
            }
        }

        details.push_str("</ul>");
        self.details_label().set_text(&QString::from_std_str(&details));
    }

    pub unsafe fn clear_detailed_view(&self) {
        self.details_label().set_text(&QString::from_std_str(String::new()));
    }

    pub unsafe fn list_selection(&self) -> Vec<Ref<QModelIndex>> {
        let indexes_visual = self.profiles_list_view().selection_model().selection().indexes();
        let mut indexes_visual = (0..indexes_visual.count_0a())
            .filter(|x| indexes_visual.at(*x).column() == 0)
            .map(|x| indexes_visual.at(x))
            .collect::<Vec<_>>();

        // Manually sort the selection, because if the user selects with ctrl from bottom to top, this breaks hard.
        indexes_visual.sort_by_key(|index| index.row());
        indexes_visual.reverse();

        indexes_visual
    }

    pub unsafe fn rename_dialog(&self, current_name: &str, in_use_names: &[String]) -> Result<Option<String>> {
        let in_use_names = in_use_names.to_vec();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { RENAME_VIEW_DEBUG } else { RENAME_VIEW_RELEASE };
        let main_widget = load_template(self.dialog(), template_path)?;

        let dialog = main_widget.static_downcast::<QDialog>();
        dialog.set_window_title(&qtr("profile_rename"));

        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "name_label")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        name_line_edit.set_text(&QString::from_std_str(current_name));
        name_label.set_text(&qtr("profile_name"));

        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        // Do not allow entering already used names.
        if in_use_names.iter().any(|name| *name == current_name) {
            button_box.button(StandardButton::Ok).set_enabled(false);
        }

        name_line_edit.text_changed().connect(&qt_core::SlotNoArgs::new(&name_line_edit, clone!(in_use_names => move || {
            let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit").unwrap();
            button_box.button(StandardButton::Ok).set_enabled(!in_use_names.contains(&&name_line_edit.text().to_std_string()));
        })));

        if dialog.exec() == 1 {
            Ok(Some(name_line_edit.text().to_std_string())) }
        else {
            Ok(None)
        }
    }

    pub unsafe fn rename_profile(&self, app_ui: &Rc<AppUI>) -> Result<()> {
        let selection = self.list_selection();
        let index = &selection[0];
        let current_name = index.data_1a(2).to_string().to_std_string();

        let names_in_use = app_ui.game_profiles().read().unwrap().keys().cloned().collect::<Vec<_>>();

        if let Some(new_name) = self.rename_dialog(&current_name, &names_in_use)? {

            if names_in_use.iter().any(|name| **name == new_name) {
                return Err(anyhow!("Name invalid, as there's already another profile with it."));
            }

            // Update the list. We need to re-select to avoid nullptr issues, though we didn't made a change that could cause it...
            let selection = self.list_selection();
            let index = &selection[0];
            let item = self.profiles_list_model().item_from_index(*index);
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&new_name)), 2);

            // Update the profile itself, and save it to disk.
            {
                let mut profiles = app_ui.game_profiles().write().unwrap();
                if let Some(mut profile) = profiles.remove(&current_name) {
                    let game = app_ui.game_selected().read().unwrap();
                    let old_profile = profile.clone();
                    old_profile.delete(&game)?;

                    profile.set_id(new_name.to_owned());
                    profile.save(&game, &new_name)?;

                    profiles.insert(new_name.to_owned(), profile);
                }
            }

            // Reload the detailed view to reflect the name change.
            let selection = self.profiles_list_view().selection_model().selection();
            self.profiles_list_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
            self.profiles_list_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
        }

        Ok(())
    }

    pub unsafe fn delete_profile(&self, app_ui: &Rc<AppUI>) -> Result<()> {
        if app_ui.are_you_sure("are_you_sure_delete_profile") {
            let selection = self.list_selection();
            let index = &selection[0];
            let name = index.data_1a(2).to_string().to_std_string();
            let row = index.row();

            // Remove it from the view.
            let selection_model = self.profiles_list_view().selection_model();
            let selection = selection_model.selection();
            self.profiles_list_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
            self.profiles_list_model().remove_row_1a(row);

            // Remove it from the backend.
            if let Some(profile) = app_ui.game_profiles().write().unwrap().remove(&name) {
                let game = app_ui.game_selected().read().unwrap();
                profile.delete(&game)?;
            }
        }

        Ok(())
    }

    pub unsafe fn create_shortcut(&self, app_ui: &Rc<AppUI>) -> Result<()> {
        let selection = self.list_selection();
        let index = &selection[0];
        let current_name = index.data_1a(2).to_string().to_std_string();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { SHORTCUT_VIEW_DEBUG } else { SHORTCUT_VIEW_RELEASE };
        let main_widget = load_template(self.dialog(), template_path)?;

        let dialog = main_widget.static_downcast::<QDialog>();
        dialog.set_window_title(&qtr("profile_shortcut"));

        let name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "name_label")?;
        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let location_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "location_label")?;
        let location_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "location_line_edit")?;
        let location_search_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "location_search_button")?;
        let game_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "game_label")?;
        let game_next_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "game_next_label")?;
        let autostart_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "autostart_label")?;
        let autostart_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "autostart_checkbox")?;

        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        name_label.set_text(&qtr("profile_shortcut_name"));
        location_label.set_text(&qtr("profile_shortcut_location"));
        game_label.set_text(&qtr("profile_shortcut_game"));
        autostart_label.set_text(&qtr("profile_shortcut_autostart"));
        name_line_edit.set_text(&QString::from_std_str(&current_name));
        game_next_label.set_text(&QString::from_std_str(app_ui.game_selected().read().unwrap().key()));
        button_box.button(StandardButton::Ok).set_enabled(false);

        // Only allow creating shortcuts if we have a name and a location. The rest is optional.
        let main_ptr = main_widget.static_upcast();
        let allow_create_slot = SlotNoArgs::new(&main_widget, move || {
            let name_line_edit: QPtr<QLineEdit> = find_widget(&main_ptr, "name_line_edit").unwrap();
            let location_line_edit: QPtr<QLineEdit> = find_widget(&main_ptr, "location_line_edit").unwrap();
            let button_box: QPtr<QDialogButtonBox> = find_widget(&main_ptr, "button_box").unwrap();
            let location_folder = PathBuf::from(location_line_edit.text().to_std_string());

            button_box.button(StandardButton::Ok).set_enabled(!name_line_edit.text().is_empty() && !location_line_edit.text().is_empty() && location_folder.is_dir());
        });

        // Slot for the location search dialog.
        let main_ptr = main_widget.static_upcast();
        let location_search_slot = SlotNoArgs::new(&main_widget, move || {
            let location_line_edit: QPtr<QLineEdit> = find_widget(&main_ptr, "location_line_edit").unwrap();

            let file_dialog = QFileDialog::from_q_widget_q_string(
                &location_line_edit,
                &qtr("select_location_folder"),
            );

            file_dialog.set_file_mode(FileMode::Directory);
            file_dialog.set_options(QFlags::from(QFileDialogOption::ShowDirsOnly));

            // If said path is not empty, and is a dir, set it as the initial directory.
            let old_path = location_line_edit.text().to_std_string();
            if !old_path.is_empty() && Path::new(&old_path).is_dir() {
                file_dialog.set_directory_q_string(&location_line_edit.text());
            }

            if file_dialog.exec() == 1 {
                let selected_files = file_dialog.selected_files();
                let path = selected_files.at(0);
                location_line_edit.set_text(path);
            }
        });

        name_line_edit.text_changed().connect(&allow_create_slot);
        location_line_edit.text_changed().connect(&allow_create_slot);
        location_search_button.released().connect(&location_search_slot);

        if dialog.exec() == 1 {
            if cfg!(target_os = "windows") {
                let mut arguments = vec![];
                arguments.push(format!("--game {}", app_ui.game_selected().read().unwrap().key()));
                arguments.push(format!("--profile {}", current_name));

                if autostart_checkbox.is_checked() {
                    arguments.push("--autostart".to_owned());
                }

                let target = std::env::current_exe()?;
                let lnk = PathBuf::from(location_line_edit.text().to_std_string()).join(format!("{}.lnk", name_line_edit.text().to_std_string()));
                let mut sl = ShellLink::new(target)?;
                sl.set_arguments(Some(arguments.join(" ")));
                sl.create_lnk(lnk)?;
            } else {
                return Err(anyhow!("Unsupported OS."))
            }
        }

        Ok(())
    }
}
