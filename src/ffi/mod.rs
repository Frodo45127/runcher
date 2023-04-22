//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QMainWindow;

use qt_core::QBox;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;

use cpp_core::Ptr;

//---------------------------------------------------------------------------//
// Custom delegates stuff.
//---------------------------------------------------------------------------//

// This function setup the special filter used for the PackFile Contents `TreeView`.
extern "C" { fn mod_list_filter(parent: *mut QObject) -> *mut QSortFilterProxyModel; }
pub fn mod_list_filter_safe(parent: QPtr<QObject>) ->  QBox<QSortFilterProxyModel> {
    unsafe { QBox::from_raw(mod_list_filter(parent.as_mut_raw_ptr())) }
}

// This function triggers the special filter used for the PackFile Contents `TreeView`. It has to be triggered here to work properly.
extern "C" { fn mod_list_trigger_filter(filter: *const QSortFilterProxyModel, pattern: *mut QRegExp); }
pub fn mod_list_trigger_filter_safe(filter: &QSortFilterProxyModel, pattern: &Ptr<QRegExp>) {
    unsafe { mod_list_trigger_filter(filter, pattern.as_mut_raw_ptr()); }
}

// This function allow us to create a custom window.
extern "C" { fn launcher_window(use_dark_theme: bool) -> *mut QMainWindow; }
pub fn launcher_window_safe(use_dark_theme: bool) -> QBox<QMainWindow> {
    unsafe { QBox::from_raw(launcher_window(use_dark_theme)) }
}

extern "C" { fn html_item_delegate(view: *mut QObject, column: i32); }
pub fn html_item_delegate_safe(view: &Ptr<QObject>, column: i32) {
    unsafe { html_item_delegate(view.as_mut_raw_ptr(), column) }
}

extern "C" { fn flags_item_delegate(view: *mut QObject, column: i32); }
pub fn flags_item_delegate_safe(view: &Ptr<QObject>, column: i32) {
    unsafe { flags_item_delegate(view.as_mut_raw_ptr(), column) }
}
