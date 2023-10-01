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

use std::path::PathBuf;
use std::rc::Rc;

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
    open_in_explorer: QBox<SlotNoArgs>,
    open_in_steam: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ModListUISlots {
    pub unsafe fn new(view: &Rc<ModListUI>) -> Self {

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
            let all_categories = !selection.is_empty() && selection.iter().all(|index| index.data_1a(VALUE_IS_CATEGORY).to_bool());
            let all_mods = !selection.is_empty() && selection.iter().all(|index| !index.data_1a(VALUE_IS_CATEGORY).to_bool());

            view.category_delete.set_enabled(all_categories);
            view.category_rename.set_enabled(all_categories && selection.len() == 1);
            view.categories_send_to_menu.set_enabled(all_mods);

            view.open_in_explorer.set_enabled(all_mods);
            view.open_in_steam.set_enabled(all_mods);
        }));

        let category_new = SlotNoArgs::new(&view.tree_view, clone!(
            view => move || {
            match view.category_new_dialog(false) {
                Ok(name) => if let Some(name) = name {
                    let item = QStandardItem::from_q_string(&QString::from_std_str(name));
                    item.set_data_2a(&QVariant::from_bool(true), VALUE_IS_CATEGORY);
                    view.model().append_row_q_standard_item(item.into_ptr().as_mut_raw_ptr());
                },
                Err(error) => show_dialog(view.tree_view(), error, false),
            }
        }));

        let open_in_explorer = SlotNoArgs::new(&view.tree_view, clone!(
            view => move || {
            let selection = view.mod_list_selection();
            for selection in &selection {
                let mut folder_path = PathBuf::from(selection.data_1a(VALUE_PACK_PATH).to_string().to_std_string());
                folder_path.pop();
                let _ = open::that(folder_path);
            }
        }));

        let open_in_steam = SlotNoArgs::new(&view.tree_view, clone!(
            view => move || {
            let selection = view.mod_list_selection();
            for selection in &selection {
                let url = selection.data_1a(VALUE_MOD_STEAM_ID).to_string().to_std_string();
                if !url.is_empty() {
                    let _ = open::that("https://steamcommunity.com/sharedfiles/filedetails/?id=".to_string() + &url);
                }
            }
        }));

        Self {
            filter_line_edit,
            filter_case_sensitive_button,
            filter_trigger,

            context_menu,
            context_menu_enabler,
            category_new,
            open_in_explorer,
            open_in_steam,
        }
    }
}
