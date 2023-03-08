//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QVariant;
use qt_core::SortOrder;
use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QTableView;
use qt_widgets::QToolButton;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::CheckState;
use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;

use anyhow::Result;
use getset::*;
use rpfm_lib::files::pack::Pack;

use std::path::Path;
use std::sync::Arc;

use rpfm_lib::games::GameInfo;
use rpfm_ui_common::utils::*;

use self::slots::PackListUISlots;

mod slots;

const VIEW_DEBUG: &str = "ui_templates/filterable_table_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_table_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct PackListUI {
    table_view: QPtr<QTableView>,
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

    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Arc<Self>> {
        let layout: QPtr<QGridLayout> = main_window.central_widget().layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

        let table_view: QPtr<QTableView> = find_widget(&main_widget.static_upcast(), "table_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        let model = QStandardItemModel::new_1a(&main_widget);
        let filter = QSortFilterProxyModel::new_1a(&main_widget);
        filter.set_source_model(&model);
        model.set_parent(&table_view);
        table_view.set_model(&filter);

        let filter_timer = QTimer::new_1a(&main_widget);
        filter_timer.set_single_shot(true);

        layout.add_widget_5a(&main_widget, 1, 1, 1, 1);

        let list = Arc::new(Self {
            table_view,
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

    pub unsafe fn load(&self, game: &GameInfo, game_path: &Path) -> Result<()> {

        let data_paths = game.data_packs_paths(game_path);
        let content_paths = game.content_packs_paths(game_path);

        let vanilla_paths = game.ca_packs_paths(game_path)?;

        if let Some(ref paths) = data_paths {
            for path in paths {
                let row = QListOfQStandardItem::new();
                let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let item_name = QStandardItem::from_q_string(&QString::from_std_str(&pack_name));
                let pack = Pack::read_and_merge(&[path.to_path_buf()], true, false)?;
                let combined_name = format!("{}{}", pack.pfh_file_type() as u32, pack_name);
                dbg!(&combined_name);
                item_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(combined_name)), 20);
                // Vanilla packs are not checkable.
                // TODO: Remove the non-loaded language files from the list.
                if !vanilla_paths.contains(path) {
                    item_name.set_checkable(true);
                    item_name.set_check_state(CheckState::Checked);
                }

                row.append_q_standard_item(&item_name.into_ptr().as_mut_raw_ptr());

                self.model().append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());
            }
        }

        if let Some(ref paths) = content_paths {
            for path in paths {
                let row = QListOfQStandardItem::new();
                let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let item_name = QStandardItem::from_q_string(&QString::from_std_str(&pack_name));
                let pack = Pack::read_and_merge(&[path.to_path_buf()], true, false)?;
                let combined_name = format!("{}{}", pack.pfh_file_type() as u32, pack_name);
                dbg!(&combined_name);
                item_name.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(combined_name)), 20);

                item_name.set_checkable(true);

                row.append_q_standard_item(&item_name.into_ptr().as_mut_raw_ptr());

                self.model().append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());
            }
        }

        // Sort first by pack type, then by ascii order.
        self.filter().set_sort_role(20);
        self.filter().sort_2a(0, SortOrder::AscendingOrder);

        self.setup_columns();

        Ok(())
    }

    pub unsafe fn setup_columns(&self) {
        let pack_name = QStandardItem::from_q_string(&QString::from_std_str("Pack Name"));

        self.model.set_horizontal_header_item(0, pack_name.into_ptr());
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
}
