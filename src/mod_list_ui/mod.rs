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
use qt_core::QFlags;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;
use qt_core::SortOrder;

use cpp_core::CppBox;
use cpp_core::Ptr;

use anyhow::Result;
use getset::*;
use time::OffsetDateTime;

use std::sync::Arc;
use std::time::UNIX_EPOCH;

use rpfm_ui_common::SLASH_DMY_DATE_FORMAT;
use rpfm_ui_common::locale::*;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::ffi::*;
use crate::integrations::GameConfig;

use self::slots::ModListUISlots;

mod slots;

const VIEW_DEBUG: &str = "ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

const CATEGORY_NEW_VIEW_DEBUG: &str = "ui_templates/category_new_dialog.ui";
const CATEGORY_NEW_VIEW_RELEASE: &str = "ui/category_new_dialog.ui";

pub const VALUE_MOD_ID: i32 = 21;
pub const VALUE_PACK_PATH: i32 = 22;
pub const VALUE_MOD_STEAM_ID: i32 = 23;
pub const VALUE_IS_CATEGORY: i32 = 40;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ModListUI {
    tree_view: QPtr<QTreeView>,
    model: QBox<QStandardItemModel>,
    filter: QBox<QSortFilterProxyModel>,
    filter_line_edit: QPtr<QLineEdit>,
    filter_case_sensitive_button: QPtr<QToolButton>,
    filter_timer: QBox<QTimer>,

    context_menu: QBox<QMenu>,
    category_new: QPtr<QAction>,
    category_delete: QPtr<QAction>,
    categories_send_to_menu: QBox<QMenu>,

    open_in_explorer: QPtr<QAction>,
    open_in_steam: QPtr<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ModListUI {

    pub unsafe fn new(parent: &QBox<QWidget>) -> Result<Arc<Self>> {
        let layout: QPtr<QGridLayout> = parent.layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        let tree_view: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        let model = QStandardItemModel::new_1a(&main_widget);
        let filter = mod_list_filter_safe(main_widget.static_upcast());
        filter.set_source_model(&model);
        model.set_parent(&tree_view);
        tree_view.set_model(&filter);

        let filter_timer = QTimer::new_1a(&main_widget);
        filter_timer.set_single_shot(true);

        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        // Context menu.
        let context_menu = QMenu::from_q_widget(&main_widget);
        let category_new = context_menu.add_action_q_string(&qtr("category_new"));
        let category_delete = context_menu.add_action_q_string(&qtr("category_delete"));
        let categories_send_to_menu = QMenu::from_q_string(&qtr("categories_send_to_menu"));
        context_menu.add_menu_q_menu(&categories_send_to_menu);

        let open_in_explorer = context_menu.add_action_q_string(&qtr("open_in_explorer"));
        let open_in_steam = context_menu.add_action_q_string(&qtr("open_in_steam"));
        context_menu.insert_separator(&open_in_explorer);

        let list = Arc::new(Self {
            tree_view,
            model,
            filter,
            filter_line_edit,
            filter_case_sensitive_button,
            filter_timer,

            context_menu,
            category_new,
            category_delete,
            categories_send_to_menu,
            open_in_explorer,
            open_in_steam
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

        self.category_new().triggered().connect(slots.category_new());
        self.open_in_explorer().triggered().connect(slots.open_in_explorer());
        self.open_in_steam().triggered().connect(slots.open_in_steam());
    }

    pub unsafe fn load(&self, game_config: &GameConfig) -> Result<()> {
        for modd in game_config.mods().values() {

            if !modd.paths().is_empty() {
                let category = QString::from_std_str(modd.category().clone().unwrap_or("Unassigned".to_owned()));
                let mut parent = None;

                // Find the parent category.
                for index in 0..self.model().row_count_0a() {
                    let item = self.model().item_1a(index);
                    if !item.is_null() && item.text().compare_q_string(&category) == 0 {
                        parent = Some(item);
                        break;
                    }
                }

                // If no parent is found, create the category parent.
                if parent.is_none() {
                    let item = QStandardItem::from_q_string(&category);
                    item.set_data_2a(&QVariant::from_bool(true), VALUE_IS_CATEGORY);
                    item.set_editable(false);
                    self.model().append_row_q_standard_item(item.into_ptr().as_mut_raw_ptr());

                    parent = Some(self.model().item_1a(self.model().row_count_0a() - 1))
                }

                if let Some(ref parent) = parent {
                    let row = QListOfQStandardItem::new();
                    //let pack = Pack::read_and_merge(&[modd.pack().to_path_buf()], true, false)?;

                    let item_mod_name = QStandardItem::new();
                    let item_creator = QStandardItem::new();
                    let item_file_size = QStandardItem::new();
                    let item_file_url = QStandardItem::new();
                    let item_preview_url = QStandardItem::new();
                    let item_time_created = QStandardItem::new();
                    let item_time_updated = QStandardItem::new();
                    let item_last_check = QStandardItem::new();

                    // TODO: make this use <b> and <i>
                    let mod_name = if modd.name() != modd.id() {
                        format!("<b>{}</b> <i>({})</i>", modd.name(), modd.id())
                    } else {
                        format!("<i>{}</i>", modd.name())
                    };

                    // TODO: show discrepancies between steam's reported data and real data.
                    let mod_size = if *modd.file_size() != 0 {
                        format!("{:.2} MB", *modd.file_size() as f64 / 8.0 / 1024.0 / 1024.0)
                    } else {
                        let size = modd.paths()[0].metadata().unwrap().len();
                        format!("{:.2} MB", size as f64 / 8.0 / 1024.0 / 1024.0)
                    };

                    let time_created = if *modd.time_created() != 0 {
                        OffsetDateTime::from_unix_timestamp(*modd.time_created() as i64).unwrap().format(&SLASH_DMY_DATE_FORMAT).unwrap()
                    } else {
                        let date = modd.paths()[0].metadata().unwrap().created().unwrap().duration_since(UNIX_EPOCH).unwrap();
                        OffsetDateTime::from_unix_timestamp(date.as_secs() as i64).unwrap().format(&SLASH_DMY_DATE_FORMAT).unwrap()
                    };

                    let time_updated = if *modd.time_updated() != 0 {
                        OffsetDateTime::from_unix_timestamp(*modd.time_updated() as i64).unwrap().format(&SLASH_DMY_DATE_FORMAT).unwrap().to_string()
                    } else {
                        "-".to_string()
                    };

                    item_mod_name.set_text(&QString::from_std_str(mod_name));
                    item_creator.set_text(&QString::from_std_str(modd.creator_name()));
                    item_file_size.set_text(&QString::from_std_str(&mod_size));
                    item_file_url.set_text(&QString::from_std_str(modd.file_url()));
                    item_preview_url.set_text(&QString::from_std_str(modd.preview_url()));
                    item_time_created.set_text(&QString::from_std_str(&time_created));
                    item_time_updated.set_text(&QString::from_std_str(&time_updated));
                    item_last_check.set_text(&QString::from_std_str(modd.last_check().to_string()));

                    item_mod_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(modd.id())), VALUE_MOD_ID);
                    item_mod_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(modd.paths()[0].to_string_lossy())), VALUE_PACK_PATH);

                    if let Some(steam_id) = modd.steam_id() {
                        item_mod_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(steam_id)), VALUE_MOD_STEAM_ID);
                    }

                    item_mod_name.set_data_2a(&QVariant::from_bool(false), VALUE_IS_CATEGORY);
                    item_mod_name.set_checkable(true);

                    item_mod_name.set_editable(false);
                    item_creator.set_editable(false);
                    item_file_size.set_editable(false);
                    item_file_url.set_editable(false);
                    item_preview_url.set_editable(false);
                    item_time_created.set_editable(false);
                    item_time_updated.set_editable(false);
                    item_last_check.set_editable(false);

                    item_file_size.set_text_alignment(AlignmentFlag::AlignVCenter | AlignmentFlag::AlignRight);

                    if *modd.enabled() {
                        item_mod_name.set_check_state(CheckState::Checked);
                    }

                    //if !modd.description().is_empty() {
                    //    if modd.description().contains("for all regions") {
                    //        println!("{}", parse_to_html(modd.description()));
                    //    }
                    //    item.set_tool_tip(&QString::from_std_str(parse_to_html(modd.description())));
                    //}

                    row.append_q_standard_item(&item_mod_name.into_ptr().as_mut_raw_ptr());
                    row.append_q_standard_item(&item_creator.into_ptr().as_mut_raw_ptr());
                    row.append_q_standard_item(&item_file_size.into_ptr().as_mut_raw_ptr());
                    row.append_q_standard_item(&item_file_url.into_ptr().as_mut_raw_ptr());
                    row.append_q_standard_item(&item_preview_url.into_ptr().as_mut_raw_ptr());
                    row.append_q_standard_item(&item_time_created.into_ptr().as_mut_raw_ptr());
                    row.append_q_standard_item(&item_time_updated.into_ptr().as_mut_raw_ptr());
                    row.append_q_standard_item(&item_last_check.into_ptr().as_mut_raw_ptr());
                    parent.append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());

                }
            }
        }

        self.setup_columns();

        self.tree_view().hide_column(3);
        self.tree_view().hide_column(4);
        self.tree_view().hide_column(7);

        // If we have no api key, don't show the author column, as we cannot get it without api key.
        if setting_string("steam_api_key").is_empty() {
            self.tree_view().hide_column(1);
        }

        self.tree_view().expand_all();
        self.tree_view().sort_by_column_2a(0, SortOrder::AscendingOrder);
        self.tree_view().header().resize_sections(ResizeMode::ResizeToContents);

        Ok(())
    }

    pub unsafe fn setup_columns(&self) {
        let item_mod_name = QStandardItem::from_q_string(&qtr("mod_name"));
        let item_creator = QStandardItem::from_q_string(&qtr("creator"));
        let item_file_size = QStandardItem::from_q_string(&qtr("file_size"));
        let item_file_url = QStandardItem::from_q_string(&qtr("file_url"));
        let item_preview_url = QStandardItem::from_q_string(&qtr("preview_url"));
        let item_time_created = QStandardItem::from_q_string(&qtr("time_created"));
        let item_time_updated = QStandardItem::from_q_string(&qtr("time_updated"));
        let item_last_check = QStandardItem::from_q_string(&qtr("last_check"));

        self.model.set_horizontal_header_item(0, item_mod_name.into_ptr());
        self.model.set_horizontal_header_item(1, item_creator.into_ptr());
        self.model.set_horizontal_header_item(2, item_file_size.into_ptr());
        self.model.set_horizontal_header_item(3, item_file_url.into_ptr());
        self.model.set_horizontal_header_item(4, item_preview_url.into_ptr());
        self.model.set_horizontal_header_item(5, item_time_created.into_ptr());
        self.model.set_horizontal_header_item(6, item_time_updated.into_ptr());
        self.model.set_horizontal_header_item(7, item_last_check.into_ptr());

        html_item_delegate_safe(&self.tree_view().static_upcast::<QObject>().as_ptr(), 0);
    }


    pub unsafe fn category_new_dialog(&self) -> Result<Option<String>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { CATEGORY_NEW_VIEW_DEBUG } else { CATEGORY_NEW_VIEW_RELEASE };
        let main_widget = load_template(self.tree_view(), template_path)?;

        let dialog = main_widget.static_downcast::<QDialog>();
        dialog.set_window_title(&qtr("category_new"));

        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "name_label")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        name_line_edit.set_text(&qtr("category_new_placeholder"));
        name_label.set_text(&qtr("category_name"));

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
        let indexes_visual = (0..indexes_visual.count_0a()).rev().map(|x| indexes_visual.at(x)).collect::<Vec<_>>();
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
}
