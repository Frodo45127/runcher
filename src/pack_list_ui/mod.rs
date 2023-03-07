//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QSortFilterProxyModel;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItemModel;
use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QTableView;
use qt_widgets::QToolButton;

use qt_gui::QStandardItem;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;

use anyhow::Result;
use getset::*;

use std::path::Path;

use rpfm_lib::games::GameInfo;
use rpfm_ui_common::utils::*;

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
    filter_autoexpand_matches_button: QPtr<QToolButton>,
    filter_case_sensitive_button: QPtr<QToolButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PackListUI {

    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Self> {
        let layout: QPtr<QGridLayout> = main_window.central_widget().layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

        let table_view: QPtr<QTableView> = find_widget(&main_widget.static_upcast(), "table_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_autoexpand_matches_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_autoexpand_matches_button")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        let model = QStandardItemModel::new_1a(&main_widget);
        let filter = QSortFilterProxyModel::new_1a(&main_widget);
        filter.set_source_model(&model);
        model.set_parent(&table_view);
        table_view.set_model(&filter);

        layout.add_widget_5a(&main_widget, 0, 1, 1, 1);

        Ok(Self {
            table_view,
            model,
            filter,
            filter_line_edit,
            filter_autoexpand_matches_button,
            filter_case_sensitive_button,
        })
    }

    pub unsafe fn load(&self, game: &GameInfo, game_path: &Path) {

        let data_paths = game.data_packs_paths(game_path);
        //let content_paths = game.content_packs_paths(game_path);
        if let Some(ref paths) = data_paths {
            for path in paths {
                let row = QListOfQStandardItem::new();
                let item_enabled = QStandardItem::new();
                item_enabled.set_checkable(true);

                let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let item_name = QStandardItem::from_q_string(&QString::from_std_str(pack_name));

                row.append_q_standard_item(&item_enabled.into_ptr().as_mut_raw_ptr());
                row.append_q_standard_item(&item_name.into_ptr().as_mut_raw_ptr());

                self.model().append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());
            }
        }

        //if let Some(ref paths) = content_paths {}
    }
}
