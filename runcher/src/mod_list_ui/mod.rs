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
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QGridLayout;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMenu;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::AlignmentFlag;
use qt_core::CaseSensitivity;
use qt_core::CheckState;
use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;

use cpp_core::CppBox;
use cpp_core::CppDeletable;
use cpp_core::Ptr;

use anyhow::Result;
use base64::prelude::*;
use getset::*;
use time::OffsetDateTime;

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::UNIX_EPOCH;

use rpfm_lib::games::GameInfo;
use rpfm_lib::utils::path_to_absolute_string;

use rpfm_ui_common::locale::*;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::ffi::*;
use crate::mod_manager::{game_config::GameConfig, icon_data, mods::Mod, secondary_mods_path};
use crate::settings_ui::last_game_update_date;

use self::slots::ModListUISlots;

mod slots;

const VIEW_DEBUG: &str = "ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

const CATEGORY_NEW_VIEW_DEBUG: &str = "ui_templates/category_new_dialog.ui";
const CATEGORY_NEW_VIEW_RELEASE: &str = "ui/category_new_dialog.ui";

pub const VALUE_MOD_ID: i32 = 21;
pub const VALUE_PACK_PATH: i32 = 22;
pub const VALUE_MOD_STEAM_ID: i32 = 23;
pub const VALUE_PACK_TYPE: i32 = 24;
pub const VALUE_TIMESTAMP: i32 = 30;
pub const VALUE_IS_CATEGORY: i32 = 40;

pub const FLAG_MOD_IS_OUTDATED: i32 = 31;
pub const FLAG_MOD_DATA_IS_OLDER_THAN_SECONDARY: i32 = 32;
pub const FLAG_MOD_DATA_IS_OLDER_THAN_CONTENT: i32 = 33;
pub const FLAG_MOD_SECONDARY_IS_OLDER_THAN_CONTENT: i32 = 34;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ModListUI {
    tree_view: QPtr<QTreeView>,
    model: QPtr<QStandardItemModel>,
    filter: QBox<QSortFilterProxyModel>,
    filter_line_edit: QPtr<QLineEdit>,
    filter_case_sensitive_button: QPtr<QToolButton>,
    filter_timer: QBox<QTimer>,

    context_menu: QBox<QMenu>,
    category_new: QPtr<QAction>,
    category_delete: QPtr<QAction>,
    category_rename: QPtr<QAction>,
    category_sort: QPtr<QAction>,
    categories_send_to_menu: QBox<QMenu>,
    enable_selected: QPtr<QAction>,
    disable_selected: QPtr<QAction>,
    expand_all: QPtr<QAction>,
    collapse_all: QPtr<QAction>,

    open_in_explorer: QPtr<QAction>,
    open_in_steam: QPtr<QAction>,
    open_in_tool_menu: QBox<QMenu>,

    upload_to_workshop: QPtr<QAction>,
    download_from_workshop: QPtr<QAction>,

    copy_to_secondary: QPtr<QAction>,
    move_to_secondary: QPtr<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ModListUI {

    pub unsafe fn new(parent: &QBox<QWidget>) -> Result<Rc<Self>> {
        let layout: QPtr<QGridLayout> = parent.layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        let tree_view_placeholder: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let tree_view = new_mod_list_tree_view_safe(main_widget.static_upcast());
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        // Replace the placeholder widget.
        let main_layout: QPtr<QGridLayout> = main_widget.layout().static_downcast();
        main_layout.replace_widget_2a(&tree_view_placeholder, &tree_view);
        tree_view_placeholder.delete();

        let model = new_mod_list_model_safe(tree_view.static_upcast());
        let filter = mod_list_filter_safe(main_widget.static_upcast());
        filter.set_source_model(&model);
        model.set_parent(&tree_view);
        tree_view.set_model(&filter);

        let filter_timer = QTimer::new_1a(&main_widget);
        filter_timer.set_single_shot(true);

        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        // Context menu.
        let context_menu = QMenu::from_q_widget(&main_widget);
        let enable_selected = context_menu.add_action_q_string(&qtr("enable_selected"));
        let disable_selected = context_menu.add_action_q_string(&qtr("disable_selected"));

        let category_new = context_menu.add_action_q_string(&qtr("category_new"));
        let category_delete = context_menu.add_action_q_string(&qtr("category_delete"));
        let category_rename = context_menu.add_action_q_string(&qtr("category_rename"));
        let category_sort = context_menu.add_action_q_string(&qtr("category_sort"));
        let categories_send_to_menu = QMenu::from_q_string(&qtr("categories_send_to_menu"));
        context_menu.add_menu_q_menu(&categories_send_to_menu);

        let open_in_explorer = context_menu.add_action_q_string(&qtr("open_in_explorer"));
        let open_in_steam = context_menu.add_action_q_string(&qtr("open_in_steam"));
        let open_in_tool_menu = QMenu::from_q_string(&qtr("open_in_tool_menu"));
        open_in_tool_menu.set_enabled(false);
        context_menu.add_menu_q_menu(&open_in_tool_menu);
        context_menu.insert_separator(&category_new);
        context_menu.insert_separator(&open_in_explorer);

        let upload_to_workshop = context_menu.add_action_q_string(&qtr("upload_to_workshop"));
        let download_from_workshop = context_menu.add_action_q_string(&qtr("download_from_workshop"));
        context_menu.insert_separator(&upload_to_workshop);

        let copy_to_secondary = context_menu.add_action_q_string(&qtr("copy_to_secondary"));
        let move_to_secondary = context_menu.add_action_q_string(&qtr("move_to_secondary"));
        context_menu.insert_separator(&copy_to_secondary);

        let expand_all = context_menu.add_action_q_string(&qtr("expand_all"));
        let collapse_all = context_menu.add_action_q_string(&qtr("collapse_all"));
        context_menu.insert_separator(&expand_all);

        let list = Rc::new(Self {
            tree_view,
            model,
            filter,
            filter_line_edit,
            filter_case_sensitive_button,
            filter_timer,

            context_menu,
            category_new,
            category_delete,
            category_rename,
            category_sort,
            categories_send_to_menu,
            enable_selected,
            disable_selected,
            expand_all,
            collapse_all,

            open_in_explorer,
            open_in_steam,
            open_in_tool_menu,

            upload_to_workshop,
            download_from_workshop,

            copy_to_secondary,
            move_to_secondary,
        });

        let slots = ModListUISlots::new(&list);
        list.set_connections(&slots);

        Ok(list)
    }

    pub unsafe fn set_connections(&self, slots: &ModListUISlots) {
        self.filter_line_edit().text_changed().connect(slots.filter_line_edit());
        self.filter_case_sensitive_button().toggled().connect(slots.filter_case_sensitive_button());
        self.filter_timer().timeout().connect(slots.filter_trigger());

        self.tree_view().custom_context_menu_requested().connect(slots.context_menu());

        self.tree_view().selection_model().selection_changed().connect(slots.context_menu_enabler());
        self.context_menu().about_to_show().connect(slots.context_menu_enabler());

        self.open_in_explorer().triggered().connect(slots.open_in_explorer());
        self.open_in_steam().triggered().connect(slots.open_in_steam());
        self.expand_all().triggered().connect(slots.expand_all());
        self.collapse_all().triggered().connect(slots.collapse_all());
    }

    pub unsafe fn load(&self, game: &GameInfo, game_config: &GameConfig) -> Result<()> {
        self.model().clear();
        self.setup_columns();

        let date_format_str = setting_string("date_format");
        let date_format = time::format_description::parse(&date_format_str).unwrap();

        let game_path = setting_path(game.key());
        let game_last_update_date = last_game_update_date(game, &game_path)?;
        let game_data_path = game.data_path(&game_path)?;

        let data_path = path_to_absolute_string(&game_data_path);
        let secondary_path = path_to_absolute_string(&secondary_mods_path(game.key()).unwrap_or_else(|_| PathBuf::default()));
        let content_path = path_to_absolute_string(&game.content_path(&game_path).unwrap_or_else(|_| PathBuf::default()));

        // Initialize these here so they can be re-use.
        let outdated_icon = icon_data("outdated.png").unwrap_or_else(|_| vec![]);
        let outdated = tre("mod_outdated_description", &[&BASE64_STANDARD.encode(outdated_icon)]);

        let data_older_than_secondary_icon = icon_data("data_older_than_secondary.png").unwrap_or_else(|_| vec![]);
        let data_older_than_secondary = tre("mod_data_older_than_secondary", &[&BASE64_STANDARD.encode(data_older_than_secondary_icon)]);

        let data_older_than_content_icon = icon_data("data_older_than_content.png").unwrap_or_else(|_| vec![]);
        let data_older_than_content = tre("mod_data_older_than_content", &[&BASE64_STANDARD.encode(data_older_than_content_icon)]);

        let secondary_older_than_content_icon = icon_data("secondary_older_than_content.png").unwrap_or_else(|_| vec![]);
        let secondary_older_than_content = tre("mod_secondary_older_than_content", &[&BASE64_STANDARD.encode(secondary_older_than_content_icon)]);

        // This loads mods per category, meaning all installed mod have to be in the categories list!!!!
        for category in game_config.categories_order() {
            let item = QStandardItem::from_q_string(&QString::from_std_str(category));
            item.set_data_2a(&QVariant::from_bool(true), VALUE_IS_CATEGORY);
            item.set_editable(false);
            self.model().append_row_q_standard_item(item.into_ptr().as_mut_raw_ptr());

            if let Some(mods) = game_config.categories().get(category) {
                for mod_id in mods {
                    if let Some(modd) = game_config.mods().get(mod_id) {

                        // Ignore registered mods with no path.
                        if !modd.paths().is_empty() {
                            let category = QString::from_std_str(game_config.category_for_mod(modd.id()));
                            let mut parent = None;

                            // Find the parent category.
                            for index in 0..self.model().row_count_0a() {
                                let item = self.model().item_1a(index);
                                if !item.is_null() && item.text().compare_q_string(&category) == 0 {
                                    parent = Some(item);
                                    break;
                                }
                            }

                            if let Some(ref parent) = parent {
                                let row = QListOfQStandardItem::new();

                                let item_mod_name = Self::new_item();
                                let item_flags = Self::new_item();
                                let item_location = Self::new_item();
                                let item_creator = Self::new_item();
                                let item_type = Self::new_item();
                                let item_file_size = Self::new_item();
                                let item_time_created = Self::new_item();
                                let item_time_updated = Self::new_item();

                                let mod_name = if modd.name() != modd.id() {
                                    if !modd.file_name().is_empty() {

                                        // Map filenames are folder names which we have to turn into packs.
                                        let pack_name = if let Some(alt_name) = modd.alt_name() {
                                            alt_name.to_string()
                                        } else {
                                            modd.file_name().split('/').last().unwrap().to_owned()
                                        };

                                        format!("<b>{}</b> <i>({} - {})</i>", modd.name(), pack_name, modd.id())
                                    } else {
                                        format!("<b>{}</b> <i>({})</i>", modd.name(), modd.id())
                                    }
                                } else {
                                    format!("<i>{}</i>", modd.name())
                                };

                                // TODO: show discrepancies between steam's reported data and real data.
                                let mod_size = if *modd.file_size() != 0 {
                                    format!("{:.2} MB", *modd.file_size() as f64 / 1024.0 / 1024.0)
                                } else {
                                    let size = modd.paths()[0].metadata()?.len();
                                    format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
                                };

                                let time_created = if *modd.time_created() != 0 {
                                    OffsetDateTime::from_unix_timestamp(*modd.time_created() as i64)?.format(&date_format)?
                                } else if cfg!(target_os = "windows") {
                                    let date = modd.paths()[0].metadata()?.created()?.duration_since(UNIX_EPOCH)?;
                                    OffsetDateTime::from_unix_timestamp(date.as_secs() as i64)?.format(&date_format)?
                                } else {
                                    String::new()
                                };

                                let time_updated = if *modd.time_updated() != 0 {
                                    OffsetDateTime::from_unix_timestamp(*modd.time_updated() as i64)?.format(&date_format)?.to_string()
                                } else {
                                    "-".to_string()
                                };

                                let mut flags_description = String::new();
                                if modd.outdated(game_last_update_date) {
                                    item_flags.set_data_2a(&QVariant::from_bool(true), FLAG_MOD_IS_OUTDATED);
                                    flags_description.push_str(&outdated);
                                }

                                if let Ok(flags) = modd.priority_dating_flags(&data_path, &secondary_path, &content_path) {
                                    item_flags.set_data_2a(&QVariant::from_bool(flags.0), FLAG_MOD_DATA_IS_OLDER_THAN_SECONDARY);
                                    item_flags.set_data_2a(&QVariant::from_bool(flags.1), FLAG_MOD_DATA_IS_OLDER_THAN_CONTENT);
                                    item_flags.set_data_2a(&QVariant::from_bool(flags.2), FLAG_MOD_SECONDARY_IS_OLDER_THAN_CONTENT);

                                    if flags.0 {
                                        flags_description.push_str(&data_older_than_secondary);
                                    }

                                    if flags.1 {
                                        flags_description.push_str(&data_older_than_content);
                                    }

                                    if flags.2 {
                                        flags_description.push_str(&secondary_older_than_content);
                                    }
                                }

                                if !flags_description.is_empty() {
                                    flags_description = tr("mod_flags_description") + "<ul>" + &flags_description + "<ul/>";
                                    item_flags.set_tool_tip(&QString::from_std_str(&flags_description));
                                }

                                let (l_data, l_secondary, l_content) = modd.location(&data_path, &secondary_path, &content_path);
                                let mut locations = vec![];

                                if l_data {
                                    locations.push("Data".to_owned());
                                }

                                if l_secondary {
                                    locations.push("Secondary".to_owned());
                                }

                                if let Some(id) = l_content {
                                    locations.push(format!("Content ({})", id));
                                }

                                item_location.set_text(&QString::from_std_str(locations.join(",")));

                                item_time_created.set_data_2a(&QVariant::from_i64(*modd.time_created() as i64), VALUE_TIMESTAMP);
                                item_time_updated.set_data_2a(&QVariant::from_i64(*modd.time_updated() as i64), VALUE_TIMESTAMP);

                                item_mod_name.set_text(&QString::from_std_str(mod_name));
                                item_creator.set_text(&QString::from_std_str(modd.creator_name()));
                                item_type.set_text(&QString::from_std_str(modd.pack_type().to_string()));
                                item_file_size.set_text(&QString::from_std_str(&mod_size));
                                item_time_created.set_text(&QString::from_std_str(&time_created));
                                item_time_updated.set_text(&QString::from_std_str(&time_updated));

                                item_mod_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(modd.id())), VALUE_MOD_ID);
                                item_mod_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(modd.paths()[0].to_string_lossy())), VALUE_PACK_PATH);

                                if let Some(steam_id) = modd.steam_id() {
                                    item_mod_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(steam_id)), VALUE_MOD_STEAM_ID);
                                }

                                item_mod_name.set_data_2a(&QVariant::from_bool(false), VALUE_IS_CATEGORY);
                                item_mod_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(modd.pack_type().to_string())), VALUE_PACK_TYPE);

                                if modd.can_be_toggled(&game_data_path) {
                                    item_mod_name.set_checkable(true);

                                    if modd.enabled(&game_data_path) {
                                        item_mod_name.set_check_state(CheckState::Checked);
                                    }
                                }

                                // This is for movie mods in /data.
                                else {
                                    item_mod_name.set_checkable(true);
                                    item_mod_name.set_check_state(CheckState::Checked);
                                    item_mod_name.set_enabled(false);
                                }

                                item_file_size.set_text_alignment(AlignmentFlag::AlignVCenter | AlignmentFlag::AlignRight);

                                row.append_q_standard_item(&item_mod_name.into_ptr().as_mut_raw_ptr());
                                row.append_q_standard_item(&item_flags.into_ptr().as_mut_raw_ptr());
                                row.append_q_standard_item(&item_location.into_ptr().as_mut_raw_ptr());
                                row.append_q_standard_item(&item_creator.into_ptr().as_mut_raw_ptr());
                                row.append_q_standard_item(&item_type.into_ptr().as_mut_raw_ptr());
                                row.append_q_standard_item(&item_file_size.into_ptr().as_mut_raw_ptr());
                                row.append_q_standard_item(&item_time_created.into_ptr().as_mut_raw_ptr());
                                row.append_q_standard_item(&item_time_updated.into_ptr().as_mut_raw_ptr());
                                parent.append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());
                            }
                        }
                    }
                }
            }
        }

        // If we have no api key, don't show the author column, as we cannot get it without api key.
        if setting_string("steam_api_key").is_empty() {
            self.tree_view().hide_column(3);
        }

        self.tree_view().expand_all();
        self.tree_view().header().resize_sections(ResizeMode::ResizeToContents);

        // Add the full flags description to the column title.
        let mut full_desc = tr("mod_flags_description") + "<ul>";
        full_desc.push_str(&outdated);
        full_desc.push_str(&data_older_than_secondary);
        full_desc.push_str(&data_older_than_content);
        full_desc.push_str(&secondary_older_than_content);
        full_desc.push_str("</ul>");

        self.model.horizontal_header_item(1).set_tool_tip(&QString::from_std_str(full_desc));

        Ok(())
    }

    pub unsafe fn update(&self, game: &GameInfo, mods: &HashMap<String, Mod>, mods_to_delete: &[String]) -> Result<()> {
        self.model().block_signals(true);

        let date_format_str = setting_string("date_format");
        let date_format = time::format_description::parse(&date_format_str).unwrap();

        let game_path = setting_path(game.key());
        let game_last_update_date = last_game_update_date(game, &game_path)?;
        let game_data_path = game.data_path(&game_path)?;

        let data_path = path_to_absolute_string(&game_data_path);
        let secondary_path = path_to_absolute_string(&secondary_mods_path(game.key()).unwrap_or_else(|_| PathBuf::default()));
        let content_path = path_to_absolute_string(&game.content_path(&game_path).unwrap_or_else(|_| PathBuf::default()));

        // Initialize these here so they can be re-use.
        let outdated_icon = icon_data("outdated.png").unwrap_or_else(|_| vec![]);
        let outdated = tre("mod_outdated_description", &[&BASE64_STANDARD.encode(outdated_icon)]);

        let data_older_than_secondary_icon = icon_data("data_older_than_secondary.png").unwrap_or_else(|_| vec![]);
        let data_older_than_secondary = tre("mod_data_older_than_secondary", &[&BASE64_STANDARD.encode(data_older_than_secondary_icon)]);

        let data_older_than_content_icon = icon_data("data_older_than_content.png").unwrap_or_else(|_| vec![]);
        let data_older_than_content = tre("mod_data_older_than_content", &[&BASE64_STANDARD.encode(data_older_than_content_icon)]);

        let secondary_older_than_content_icon = icon_data("secondary_older_than_content.png").unwrap_or_else(|_| vec![]);
        let secondary_older_than_content = tre("mod_secondary_older_than_content", &[&BASE64_STANDARD.encode(secondary_older_than_content_icon)]);

        for category_index in 0..self.model().row_count_0a() {
            let category = self.model().item_2a(category_index, 0);
            let mut index_to_delete = vec![];
            for mod_index in 0..category.row_count() {
                let item_mod_name = category.child_2a(mod_index, 0);
                let mod_id = item_mod_name.data_1a(VALUE_MOD_ID).to_string().to_std_string();

                if mods_to_delete.contains(&mod_id) {
                    index_to_delete.push(mod_index);
                    continue;
                }

                if !mod_id.is_empty() {
                    if let Some(modd) = mods.get(&mod_id) {
                        let item_flags = category.child_2a(mod_index, 1);
                        let item_location = category.child_2a(mod_index, 2);
                        let item_creator = category.child_2a(mod_index, 3);
                        let item_type = category.child_2a(mod_index, 4);
                        let item_file_size = category.child_2a(mod_index, 5);
                        let item_time_created = category.child_2a(mod_index, 6);
                        let item_time_updated = category.child_2a(mod_index, 7);

                        let mod_name = if modd.name() != modd.id() {
                            if !modd.file_name().is_empty() {

                                // Map filenames are folder names which we have to turn into packs.
                                let pack_name = if let Some(alt_name) = modd.alt_name() {
                                    alt_name.to_string()
                                } else {
                                    modd.file_name().split('/').last().unwrap().to_owned()
                                };

                                format!("<b>{}</b> <i>({} - {})</i>", modd.name(), pack_name, modd.id())
                            } else {
                                format!("<b>{}</b> <i>({})</i>", modd.name(), modd.id())
                            }
                        } else {
                            format!("<i>{}</i>", modd.name())
                        };

                        // TODO: show discrepancies between steam's reported data and real data.
                        let mod_size = if *modd.file_size() != 0 {
                            format!("{:.2} MB", *modd.file_size() as f64 / 1024.0 / 1024.0)
                        } else {
                            let size = modd.paths()[0].metadata()?.len();
                            format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
                        };

                        let time_created = if *modd.time_created() != 0 {
                            OffsetDateTime::from_unix_timestamp(*modd.time_created() as i64)?.format(&date_format)?
                        } else if cfg!(target_os = "windows") {
                            let date = modd.paths()[0].metadata()?.created()?.duration_since(UNIX_EPOCH)?;
                            OffsetDateTime::from_unix_timestamp(date.as_secs() as i64)?.format(&date_format)?
                        } else {
                            String::new()
                        };

                        let time_updated = if *modd.time_updated() != 0 {
                            OffsetDateTime::from_unix_timestamp(*modd.time_updated() as i64)?.format(&date_format)?.to_string()
                        } else {
                            "-".to_string()
                        };

                        let mut flags_description = String::new();
                        if modd.outdated(game_last_update_date) {
                            item_flags.set_data_2a(&QVariant::from_bool(true), FLAG_MOD_IS_OUTDATED);
                            flags_description.push_str(&outdated);
                        }

                        if let Ok(flags) = modd.priority_dating_flags(&data_path, &secondary_path, &content_path) {
                            item_flags.set_data_2a(&QVariant::from_bool(flags.0), FLAG_MOD_DATA_IS_OLDER_THAN_SECONDARY);
                            item_flags.set_data_2a(&QVariant::from_bool(flags.1), FLAG_MOD_DATA_IS_OLDER_THAN_CONTENT);
                            item_flags.set_data_2a(&QVariant::from_bool(flags.2), FLAG_MOD_SECONDARY_IS_OLDER_THAN_CONTENT);

                            if flags.0 {
                                flags_description.push_str(&data_older_than_secondary);
                            }

                            if flags.1 {
                                flags_description.push_str(&data_older_than_content);
                            }

                            if flags.2 {
                                flags_description.push_str(&secondary_older_than_content);
                            }
                        }

                        if !flags_description.is_empty() {
                            flags_description = tr("mod_flags_description") + "<ul>" + &flags_description + "<ul/>";
                            item_flags.set_tool_tip(&QString::from_std_str(&flags_description));
                        }

                        let (l_data, l_secondary, l_content) = modd.location(&data_path, &secondary_path, &content_path);
                        let mut locations = vec![];

                        if l_data {
                            locations.push("Data".to_owned());
                        }

                        if l_secondary {
                            locations.push("Secondary".to_owned());
                        }

                        if let Some(id) = l_content {
                            locations.push(format!("Content ({})", id));
                        }

                        item_location.set_text(&QString::from_std_str(locations.join(",")));

                        item_time_created.set_data_2a(&QVariant::from_i64(*modd.time_created() as i64), VALUE_TIMESTAMP);
                        item_time_updated.set_data_2a(&QVariant::from_i64(*modd.time_updated() as i64), VALUE_TIMESTAMP);

                        item_mod_name.set_text(&QString::from_std_str(mod_name));
                        item_creator.set_text(&QString::from_std_str(modd.creator_name()));
                        item_type.set_text(&QString::from_std_str(modd.pack_type().to_string()));
                        item_file_size.set_text(&QString::from_std_str(&mod_size));
                        item_time_created.set_text(&QString::from_std_str(&time_created));
                        item_time_updated.set_text(&QString::from_std_str(&time_updated));
                    }
                }
            }

            if !index_to_delete.is_empty() {
                index_to_delete.reverse();

                self.model().block_signals(false);

                for index in index_to_delete {
                    category.remove_row(index);
                }

                self.model().block_signals(true);
            }
        }

        self.model().block_signals(false);

        Ok(())
    }

    pub unsafe fn setup_columns(&self) {
        self.model.set_column_count(7);

        let item_mod_name = QStandardItem::from_q_string(&qtr("mod_name"));
        let item_flags = QStandardItem::from_q_string(&qtr("flags"));
        let item_location = QStandardItem::from_q_string(&qtr("location"));
        let item_creator = QStandardItem::from_q_string(&qtr("creator"));
        let item_pack_type = QStandardItem::from_q_string(&qtr("pack_type"));
        let item_file_size = QStandardItem::from_q_string(&qtr("file_size"));
        let item_time_created = QStandardItem::from_q_string(&qtr("time_created"));
        let item_time_updated = QStandardItem::from_q_string(&qtr("time_updated"));

        self.model.set_horizontal_header_item(0, item_mod_name.into_ptr());
        self.model.set_horizontal_header_item(1, item_flags.into_ptr());
        self.model.set_horizontal_header_item(2, item_location.into_ptr());
        self.model.set_horizontal_header_item(3, item_creator.into_ptr());
        self.model.set_horizontal_header_item(4, item_pack_type.into_ptr());
        self.model.set_horizontal_header_item(5, item_file_size.into_ptr());
        self.model.set_horizontal_header_item(6, item_time_created.into_ptr());
        self.model.set_horizontal_header_item(7, item_time_updated.into_ptr());

        html_item_delegate_safe(&self.tree_view().static_upcast::<QObject>().as_ptr(), 0);
        flags_item_delegate_safe(&self.tree_view().static_upcast::<QObject>().as_ptr(), 1);

        self.tree_view.header().set_minimum_section_size(24 * 4);
    }

    pub unsafe fn category_new_dialog(&self, rename: bool) -> Result<Option<String>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { CATEGORY_NEW_VIEW_DEBUG } else { CATEGORY_NEW_VIEW_RELEASE };
        let main_widget = load_template(self.tree_view(), template_path)?;

        let dialog = main_widget.static_downcast::<QDialog>();
        if rename {
            dialog.set_window_title(&qtr("category_rename"));
        } else {
            dialog.set_window_title(&qtr("category_new"));
        }

        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "name_label")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        name_line_edit.set_placeholder_text(&qtr("category_new_placeholder"));
        name_label.set_text(&qtr("category_name"));

        // If we're renaming, use the current name as the default name.
        if rename {
            let selection = self.mod_list_selection();
            let cat_index = &selection[0];
            let old_cat_name = cat_index.data_1a(2).to_string().to_std_string();
            name_line_edit.set_text(&QString::from_std_str(old_cat_name));
        }

        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        // Do not allow entering already used names.
        let categories = self.categories();
        if categories.contains(&tr("category_new_placeholder")) {
            button_box.button(StandardButton::Ok).set_enabled(false);
        }

        name_line_edit.text_changed().connect(&qt_core::SlotNoArgs::new(&name_line_edit, move || {
            let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit").unwrap();
            button_box.button(StandardButton::Ok).set_enabled(!categories.contains(&name_line_edit.text().to_std_string()));
        }));

        if dialog.exec() == 1 {
            Ok(Some(name_line_edit.text().to_std_string())) }
        else {
            Ok(None)
        }
    }

    pub unsafe fn categories(&self) -> Vec<String> {
        let mut categories = Vec::with_capacity(self.model().row_count_0a() as usize);
        for index in 0..self.model().row_count_0a() {
            let item = self.model().item_1a(index);
            if !item.is_null() {
                categories.push(item.text().to_std_string());
            }
        }

        categories
    }

    pub unsafe fn category_item(&self, category: &str) -> Option<Ptr<QStandardItem>> {
        let mut cat_item = None;
        let category = QString::from_std_str(category);
        for index in 0..self.model().row_count_0a() {
            let item = self.model().item_1a(index);
            if !item.is_null() && item.text().compare_q_string(&category) == 0 {
                cat_item = Some(item);
                break;
            }
        }

        cat_item
    }

    pub unsafe fn mod_list_selection(&self) -> Vec<CppBox<QModelIndex>> {
        let indexes_visual = self.tree_view().selection_model().selection().indexes();
        let mut indexes_visual = (0..indexes_visual.count_0a())
            .filter(|x| indexes_visual.at(*x).column() == 0)
            .map(|x| indexes_visual.at(x))
            .collect::<Vec<_>>();

        // Manually sort the selection, because if the user selects with ctrl from bottom to top, this breaks hard.
        indexes_visual.sort_by_key(|index| index.row());
        indexes_visual.reverse();

        indexes_visual.iter().map(|x| self.filter().map_to_source(*x)).collect::<Vec<_>>()
    }

    pub unsafe fn filter_list(&self) {

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&self.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = self.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        mod_list_trigger_filter_safe(self.filter(), &pattern.as_ptr());
    }

    pub unsafe fn delayed_updates(&self) {
        self.filter_timer.set_interval(500);
        self.filter_timer.start_0a();
    }

    unsafe fn new_item() -> CppBox<QStandardItem> {
        let item = QStandardItem::new();
        item.set_editable(false);
        item
    }
}
