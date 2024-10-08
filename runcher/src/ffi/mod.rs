//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QMainWindow;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::Signal;
use qt_core::QString;
use qt_core::QStringList;

use cpp_core::Ptr;

use rpfm_lib::games::supported_games::SupportedGames;

//---------------------------------------------------------------------------//
// Custom delegates stuff.
//---------------------------------------------------------------------------//

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

extern "C" { fn path_item_delegate(view: *mut QObject, column: i32); }
pub fn path_item_delegate_safe(view: &Ptr<QObject>, column: i32) {
    unsafe { path_item_delegate(view.as_mut_raw_ptr(), column) }
}

extern "C" { fn game_selector_item_delegate(view: *mut QObject, column: i32, game_keys: *const QStringList); }
pub fn game_selector_item_delegate_safe(view: &Ptr<QObject>, column: i32) {
    unsafe {
        let games = SupportedGames::default();
        let keys = games.game_keys_sorted();
        let qkeys = QStringList::new();

        for key in keys {
            qkeys.append_q_string(&QString::from_std_str(key));
        }

        game_selector_item_delegate(view.as_mut_raw_ptr(), column, qkeys.into_ptr().as_raw_ptr());
    }
}

//---------------------------------------------------------------------------//
// Drag&Drop stuff.
//---------------------------------------------------------------------------//

extern "C" { fn new_mod_list_model(parent: *mut QWidget) -> *mut QStandardItemModel; }
pub fn new_mod_list_model_safe(parent: QPtr<QWidget>) -> QPtr<QStandardItemModel> {
    unsafe { QPtr::from_raw(new_mod_list_model(parent.as_mut_raw_ptr())) }
}

// This function allow us to create a QTreeView compatible with draggable items
extern "C" { fn new_mod_list_tree_view(parent: *mut QWidget) -> *mut QTreeView; }
pub fn new_mod_list_tree_view_safe(parent: QPtr<QWidget>) -> QPtr<QTreeView> {
    unsafe { QPtr::from_raw(new_mod_list_tree_view(parent.as_mut_raw_ptr())) }
}

extern "C" { fn mod_list_filter(parent: *mut QObject) -> *mut QSortFilterProxyModel; }
pub fn mod_list_filter_safe(parent: QPtr<QObject>) ->  QBox<QSortFilterProxyModel> {
    unsafe { QBox::from_raw(mod_list_filter(parent.as_mut_raw_ptr())) }
}

extern "C" { fn mod_list_trigger_filter(filter: *const QSortFilterProxyModel, pattern: *mut QRegExp); }
pub fn mod_list_trigger_filter_safe(filter: &QSortFilterProxyModel, pattern: &Ptr<QRegExp>) {
    unsafe { mod_list_trigger_filter(filter, pattern.as_mut_raw_ptr()); }
}

extern "C" { fn new_pack_list_model(parent: *mut QWidget) -> *mut QStandardItemModel; }
pub fn new_pack_list_model_safe(parent: QPtr<QWidget>) -> QPtr<QStandardItemModel> {
    unsafe { QPtr::from_raw(new_pack_list_model(parent.as_mut_raw_ptr())) }
}

// This function allow us to create a QTreeView compatible with draggable items
extern "C" { fn new_pack_list_tree_view(parent: *mut QWidget) -> *mut QTreeView; }
pub fn new_pack_list_tree_view_safe(parent: QPtr<QWidget>) -> QPtr<QTreeView> {
    unsafe { QPtr::from_raw(new_pack_list_tree_view(parent.as_mut_raw_ptr())) }
}

extern "C" { fn pack_list_filter(parent: *mut QObject) -> *mut QSortFilterProxyModel; }
pub fn pack_list_filter_safe(parent: QPtr<QObject>) ->  QBox<QSortFilterProxyModel> {
    unsafe { QBox::from_raw(pack_list_filter(parent.as_mut_raw_ptr())) }
}

extern "C" { fn pack_list_trigger_filter(filter: *const QSortFilterProxyModel, pattern: *mut QRegExp); }
pub fn pack_list_trigger_filter_safe(filter: &QSortFilterProxyModel, pattern: &Ptr<QRegExp>) {
    unsafe { pack_list_trigger_filter(filter, pattern.as_mut_raw_ptr()); }
}

pub fn draggable_tree_view_drop_signal(widget: QPtr<QWidget>) -> Signal<(*const QModelIndex, i32)> {
    unsafe {
        Signal::new(
            ::cpp_core::Ref::from_raw(widget.as_raw_ptr()).expect("attempted to construct a null Ref"),
            ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                b"2itemDrop(QModelIndex const &,int)\0",
            ),
        )
    }
}
