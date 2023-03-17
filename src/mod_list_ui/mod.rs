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
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QMenu;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::CheckState;
use qt_core::QBox;
use qt_core::QModelIndex;
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

use std::sync::Arc;

use rpfm_ui_common::locale::*;
use rpfm_ui_common::utils::*;

use crate::ffi::*;
use crate::integrations::{GameConfig, steam::*};

use self::slots::ModListUISlots;

mod slots;

const VIEW_DEBUG: &str = "ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

const CATEGORY_NEW_VIEW_DEBUG: &str = "ui_templates/category_new_dialog.ui";
const CATEGORY_NEW_VIEW_RELEASE: &str = "ui/category_new_dialog.ui";

pub const VALUE_MOD_ID: i32 = 21;
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
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ModListUI {

    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Arc<Self>> {
        let layout: QPtr<QGridLayout> = main_window.central_widget().layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

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

        layout.add_widget_5a(&main_widget, 0, 0, 2, 1);

        // Context menu.
        let context_menu = QMenu::from_q_widget(&main_widget);
        let category_new = context_menu.add_action_q_string(&qtr("category_new"));
        let category_delete = context_menu.add_action_q_string(&qtr("category_delete"));


        let categories_send_to_menu = QMenu::from_q_string(&qtr("categories_send_to_menu"));
        context_menu.add_menu_q_menu(&categories_send_to_menu);

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
                    let item = QStandardItem::from_q_string(&QString::from_std_str(&modd.name()));
                    //let pack = Pack::read_and_merge(&[modd.pack().to_path_buf()], true, false)?;
                    item.set_checkable(true);
                    if *modd.enabled() {
                        item.set_check_state(CheckState::Checked);
                    }
                    item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(modd.id())), VALUE_MOD_ID);
                    item.set_data_2a(&QVariant::from_bool(false), VALUE_IS_CATEGORY);
                    item.set_editable(false);

                    //if !modd.description().is_empty() {
                    //    if modd.description().contains("for all regions") {
                    //        println!("{}", parse_to_html(modd.description()));
                    //    }
                    //    item.set_tool_tip(&QString::from_std_str(parse_to_html(modd.description())));
                    //}

                    row.append_q_standard_item(&item.into_ptr().as_mut_raw_ptr());
                    parent.append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());

                }
            }
        }

        self.tree_view().expand_all();
        self.tree_view().sort_by_column_2a(0, SortOrder::AscendingOrder);
        Ok(())
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
