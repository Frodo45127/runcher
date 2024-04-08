//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;

use qt_core::QBox;

use getset::*;

use std::rc::Rc;

use rpfm_ui_common::clone;
use rpfm_ui_common::utils::show_dialog;

use crate::app_ui::AppUI;

use super::ProfilesUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ProfilesUISlots {
    update_details: QBox<SlotOfQItemSelectionQItemSelection>,
    profile_rename: QBox<SlotNoArgs>,
    profile_delete: QBox<SlotNoArgs>,
    profile_shorcut: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ProfilesUISlots {

    pub unsafe fn new(ui: &Rc<ProfilesUI>, app_ui: &Rc<AppUI>) -> Self {
        let update_details = SlotOfQItemSelectionQItemSelection::new(ui.main_widget(), clone!(
            app_ui,
            ui => move |after, before| {

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    //let indexes = before.at(0).indexes();
                    //let index = indexes.at(0);
                    //view.save_entry_from_detailed_view(index.as_ref());
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let indexes = after.at(0).indexes();
                    let index = indexes.at(0);
                    ui.load_entry_to_detailed_view(&app_ui, index);

                    // Enable the buttons.
                    ui.delete_profile_button().set_enabled(true);
                    ui.rename_profile_button().set_enabled(true);
                    ui.shortcut_button().set_enabled(true);
                }

                // If nothing is loaded, means we're selecting multiple things, or none.
                // We need to clear the view to ensure no weird shenaningans happen.
                else {
                    ui.clear_detailed_view();

                    // Disable the buttons.
                    ui.delete_profile_button().set_enabled(false);
                    ui.rename_profile_button().set_enabled(false);
                    ui.shortcut_button().set_enabled(false);
                }
            }
        ));

        let profile_rename = SlotNoArgs::new(ui.main_widget(), clone!(
            app_ui,
            ui => move || {
                if let Err(error) = ui.rename_profile(&app_ui) {
                    show_dialog(ui.main_widget(), error, false);
                }
            }
        ));

        let profile_delete = SlotNoArgs::new(ui.main_widget(), clone!(
            app_ui,
            ui => move || {
                if let Err(error) = ui.delete_profile(&app_ui) {
                    show_dialog(ui.main_widget(), error, false);
                }
            }
        ));

        let profile_shorcut = SlotNoArgs::new(ui.main_widget(), clone!(
            app_ui,
            ui => move || {
                if let Err(error) = ui.create_shortcut(&app_ui) {
                    show_dialog(ui.main_widget(), error, false);
                }
            }
        ));

        Self {
            update_details,

            profile_rename,
            profile_delete,
            profile_shorcut,
        }
    }
}
