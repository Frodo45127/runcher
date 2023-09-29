//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QGridLayout;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QLineEdit;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;
use qt_core::SortOrder;

use cpp_core::CppBox;

use anyhow::Result;
use getset::*;

use std::sync::Arc;
use std::path::Path;

use rpfm_lib::files::pack::Pack;
use rpfm_lib::games::GameInfo;

use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::utils::*;

use crate::mod_manager::game_config::GameConfig;
use crate::mod_manager::load_order::LoadOrder;

use self::slots::PackListUISlots;

mod slots;

const VIEW_DEBUG: &str = "ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct PackListUI {
    tree_view: QPtr<QTreeView>,
    model: QBox<QStandardItemModel>,
    filter: QBox<QSortFilterProxyModel>,
    filter_line_edit: QPtr<QLineEdit>,
    filter_case_sensitive_button: QPtr<QToolButton>,
    filter_timer: QBox<QTimer>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PackListUI {

    pub unsafe fn new(parent: &QBox<QWidget>) -> Result<Arc<Self>> {
        let layout: QPtr<QGridLayout> = parent.layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        let tree_view: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        let model = QStandardItemModel::new_1a(&main_widget);
        let filter = QSortFilterProxyModel::new_1a(&main_widget);
        filter.set_source_model(&model);
        model.set_parent(&tree_view);
        tree_view.set_model(&filter);
        tree_view.set_sorting_enabled(false);

        let filter_timer = QTimer::new_1a(&main_widget);
        filter_timer.set_single_shot(true);

        layout.add_widget_5a(&main_widget, 1, 0, 1, 1);

        let list = Arc::new(Self {
            tree_view,
            model,
            filter,
            filter_line_edit,
            filter_case_sensitive_button,
            filter_timer,
        });

        let slots = PackListUISlots::new(&list);
        list.set_connections(&slots);

        Ok(list)
    }

    pub unsafe fn set_connections(&self, slots: &PackListUISlots) {
        self.filter_line_edit().text_changed().connect(slots.filter_line_edit());
        self.filter_case_sensitive_button().toggled().connect(slots.filter_case_sensitive_button());
        self.filter_timer().timeout().connect(slots.filter_trigger());
    }

    pub unsafe fn load(&self, game_config: &GameConfig, game_info: &GameInfo, game_path: &Path) -> Result<()> {
        self.model().clear();

        let mut load_order = LoadOrder::default();
        load_order.generate(game_config);

        if !game_path.to_string_lossy().is_empty() {
            if let Ok(game_data_folder) = game_info.data_path(game_path) {
                for (index, mod_id) in load_order.mods().iter().enumerate() {
                    if let Some(modd) = game_config.mods().get(mod_id) {

                        let row = QListOfQStandardItem::new();
                        let pack_name = modd.paths()[0].file_name().unwrap().to_string_lossy().as_ref().to_owned();
                        let pack = Pack::read_and_merge(&[modd.paths()[0].to_path_buf()], true, false)?;

                        let item_name = Self::new_item();
                        let item_type = Self::new_item();
                        let item_path = Self::new_item();
                        let load_order = Self::new_item();
                        let location = Self::new_item();
                        let steam_id = Self::new_item();

                        item_name.set_text(&QString::from_std_str(&pack_name));
                        item_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str((pack.pfh_file_type() as u32).to_string() + &pack_name)), 20);
                        item_type.set_text(&QString::from_std_str(&modd.pack_type().to_string()));
                        item_path.set_text(&QString::from_std_str(&modd.paths()[0].to_string_lossy()));
                        load_order.set_data_2a(&QVariant::from_int(index as i32), 2);

                        location.set_text(&QString::from_std_str(
                            if modd.paths()[0].starts_with(&game_data_folder) {
                                "Data".to_string()
                            } else {
                                format!("Content ({})", modd.steam_id().as_ref().unwrap())
                            }
                        ));

                        if let Some(id) = modd.steam_id() {
                            steam_id.set_text(&QString::from_std_str(id));
                        }

                        row.append_q_standard_item(&item_name.into_ptr().as_mut_raw_ptr());
                        row.append_q_standard_item(&item_type.into_ptr().as_mut_raw_ptr());
                        row.append_q_standard_item(&item_path.into_ptr().as_mut_raw_ptr());
                        row.append_q_standard_item(&load_order.into_ptr().as_mut_raw_ptr());
                        row.append_q_standard_item(&location.into_ptr().as_mut_raw_ptr());
                        row.append_q_standard_item(&steam_id.into_ptr().as_mut_raw_ptr());

                        self.model().append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());
                    }
                }
            }
        }

        self.tree_view().hide_column(2);
        self.tree_view().hide_column(5);

        self.setup_columns();
        self.tree_view().sort_by_column_2a(3, SortOrder::AscendingOrder);
        self.tree_view().header().resize_sections(ResizeMode::ResizeToContents);

        Ok(())
    }

    pub unsafe fn setup_columns(&self) {
        let pack_name = QStandardItem::from_q_string(&qtr("pack_name"));
        let pack_type = QStandardItem::from_q_string(&qtr("pack_type"));
        let pack_path = QStandardItem::from_q_string(&qtr("pack_path"));
        let load_order = QStandardItem::from_q_string(&qtr("load_order"));
        let location = QStandardItem::from_q_string(&qtr("location"));
        let steam_id = QStandardItem::from_q_string(&qtr("steam_id"));

        self.model.set_horizontal_header_item(0, pack_name.into_ptr());
        self.model.set_horizontal_header_item(1, pack_type.into_ptr());
        self.model.set_horizontal_header_item(2, pack_path.into_ptr());
        self.model.set_horizontal_header_item(3, load_order.into_ptr());
        self.model.set_horizontal_header_item(4, location.into_ptr());
        self.model.set_horizontal_header_item(5, steam_id.into_ptr());
    }

    pub unsafe fn filter_list(&self) {

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&self.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = self.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        self.filter().set_filter_reg_exp_q_reg_exp(&pattern);
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
