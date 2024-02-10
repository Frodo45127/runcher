//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::{SlotNoArgs, SlotOfQString};

use std::rc::Rc;

use rpfm_ui_common::clone;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct DataListUISlots {
    filter_line_edit: QBox<SlotOfQString>,
    filter_case_sensitive_button: QBox<SlotNoArgs>,
    filter_trigger: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl DataListUISlots {
    pub unsafe fn new(view: &Rc<DataListUI>) -> Self {

        let filter_line_edit = SlotOfQString::new(view.tree_view(), clone!(
            view => move |_| {
            view.delayed_updates();
        }));

        let filter_case_sensitive_button = SlotNoArgs::new(view.tree_view(), clone!(
            view => move || {
            view.filter_list();
        }));

        let filter_trigger = SlotNoArgs::new(view.tree_view(), clone!(
            view => move || {
            view.filter_list();
        }));

        Self {
            filter_line_edit,
            filter_case_sensitive_button,
            filter_trigger,
        }
    }
}
