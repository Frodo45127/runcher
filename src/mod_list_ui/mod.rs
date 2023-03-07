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

use qt_core::QBox;
use qt_core::QPtr;

use anyhow::Result;
use getset::*;

use rpfm_ui_common::utils::*;

const VIEW_DEBUG: &str = "ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//


#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ModListUI {
    tree_view: QPtr<QTreeView>,
    filter_line_edit: QPtr<QLineEdit>,
    filter_autoexpand_matches_button: QPtr<QToolButton>,
    filter_case_sensitive_button: QPtr<QToolButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ModListUI {

    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Self> {
        let layout: QPtr<QGridLayout> = main_window.central_widget().layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

        let tree_view: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_autoexpand_matches_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_autoexpand_matches_button")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        Ok(Self {
            tree_view,
            filter_line_edit,
            filter_autoexpand_matches_button,
            filter_case_sensitive_button,
        })
    }
}
