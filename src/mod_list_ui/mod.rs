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
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;

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

use anyhow::Result;
use getset::*;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use rpfm_lib::files::pack::Pack;
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType};

use rpfm_ui_common::utils::*;

use crate::integrations::Mod;
use self::slots::ModListUISlots;

mod slots;

const VIEW_DEBUG: &str = "ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

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
    filter_timer: QBox<QTimer>
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
        let filter = QSortFilterProxyModel::new_1a(&main_widget);
        filter.set_source_model(&model);
        model.set_parent(&tree_view);
        tree_view.set_model(&filter);

        let filter_timer = QTimer::new_1a(&main_widget);
        filter_timer.set_single_shot(true);

        layout.add_widget_5a(&main_widget, 0, 0, 2, 1);

        let list = Arc::new(Self {
            tree_view,
            model,
            filter,
            filter_line_edit,
            filter_case_sensitive_button,
            filter_timer,
        });

        let slots = ModListUISlots::new(&list);
        list.set_connections(&slots);

        Ok(list)
    }

    pub unsafe fn set_connections(&self, slots: &ModListUISlots) {
        self.filter_line_edit().text_changed().connect(slots.filter_line_edit());
        self.filter_case_sensitive_button().toggled().connect(slots.filter_case_sensitive_button());
        self.filter_timer().timeout().connect(slots.filter_trigger());
    }

    pub unsafe fn load(&self, game: &GameInfo, game_path: &Path, categories: &HashMap<String, Vec<Mod>>) -> Result<()> {
        for (category, modd) in categories {
            let category = QString::from_std_str(category);
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
                self.model().append_row_q_standard_item(item.into_ptr().as_mut_raw_ptr());

                parent = Some(self.model().item_1a(self.model().row_count_0a() - 1))
            }

            for modd in modd {
                if let Some(ref parent) = parent {
                    let row = QListOfQStandardItem::new();
                    let pack_name = modd.pack().file_name().unwrap().to_string_lossy().as_ref().to_owned();
                    let item = QStandardItem::from_q_string(&QString::from_std_str(&pack_name));
                    //let pack = Pack::read_and_merge(&[modd.pack().to_path_buf()], true, false)?;
                    item.set_checkable(true);

                    row.append_q_standard_item(&item.into_ptr().as_mut_raw_ptr());
                    parent.append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());
                }
            }
        }

        Ok(())
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
