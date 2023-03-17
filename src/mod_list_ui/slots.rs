//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::{SlotNoArgs, SlotOfQString};

use std::sync::Arc;

use rpfm_ui_common::clone;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct ModListUISlots {
    filter_line_edit: QBox<SlotOfQString>,
    filter_case_sensitive_button: QBox<SlotNoArgs>,
    filter_trigger: QBox<SlotNoArgs>,

    context_menu: QBox<SlotOfQPoint>,
    context_menu_enabler: QBox<SlotNoArgs>,

    category_new: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ModListUISlots {
    pub unsafe fn new(view: &Arc<ModListUI>) -> Self {

        let filter_line_edit = SlotOfQString::new(&view.tree_view, clone!(
            view => move |_| {
            view.delayed_updates();
        }));

        let filter_case_sensitive_button = SlotNoArgs::new(&view.tree_view, clone!(
            view => move || {
            view.filter_list();
        }));

        let filter_trigger = SlotNoArgs::new(&view.tree_view, clone!(
            view => move || {
            view.filter_list();
        }));

        let context_menu = SlotOfQPoint::new(&view.tree_view, clone!(
            view => move |_| {
            view.context_menu().exec_1a_mut(&QCursor::pos_0a());
        }));

        let context_menu_enabler = SlotNoArgs::new(&view.tree_view, clone!(
            view => move || {
            let selection = view.mod_list_selection();
            let all_categories = selection.iter().all(|index| index.data_1a(VALUE_IS_CATEGORY).to_bool());
            let all_mods = selection.iter().all(|index| !index.data_1a(VALUE_IS_CATEGORY).to_bool());

            view.category_delete.set_enabled(all_categories);
            view.categories_send_to_menu.set_enabled(all_mods);
        }));

        let category_new = SlotNoArgs::new(&view.tree_view, clone!(
            view => move || {
            match view.category_new_dialog() {
                Ok(name) => if let Some(name) = name {
                    let item = QStandardItem::from_q_string(&QString::from_std_str(name));
                    item.set_data_2a(&QVariant::from_bool(true), VALUE_IS_CATEGORY);
                    view.model().append_row_q_standard_item(item.into_ptr().as_mut_raw_ptr());
                },
                Err(error) => show_dialog(view.tree_view(), error, false),
            }
        }));

        Self {
            filter_line_edit,
            filter_case_sensitive_button,
            filter_trigger,

            context_menu,
            context_menu_enabler,
            category_new,
        }
    }
}
