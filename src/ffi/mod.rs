//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QMainWindow;
use qt_widgets::QWidget;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::Signal;

//---------------------------------------------------------------------------//
// Custom delegates stuff.
//---------------------------------------------------------------------------//
/*
// This function setup the special filter used for the PackFile Contents `TreeView`.
extern "C" { fn new_treeview_filter(parent: *mut QObject) -> *mut QSortFilterProxyModel; }
pub fn new_treeview_filter_safe(parent: QPtr<QObject>) ->  QBox<QSortFilterProxyModel> {
    unsafe { QBox::from_raw(new_treeview_filter(parent.as_mut_raw_ptr())) }
}

// This function triggers the special filter used for the PackFile Contents `TreeView`. It has to be triggered here to work properly.
extern "C" { fn trigger_treeview_filter(filter: *const QSortFilterProxyModel, pattern: *mut QRegExp); }
pub fn trigger_treeview_filter_safe(filter: &QSortFilterProxyModel, pattern: &Ptr<QRegExp>) {
    unsafe { trigger_treeview_filter(filter, pattern.as_mut_raw_ptr()); }
}

// This function allow us to create a QTreeView compatible with draggable items
extern "C" { fn new_packed_file_treeview(parent: *mut QWidget) -> *mut QTreeView; }
pub fn new_packed_file_treeview_safe(parent: QPtr<QWidget>) -> QPtr<QTreeView> {
    unsafe { QPtr::from_raw(new_packed_file_treeview(parent.as_mut_raw_ptr())) }
}

pub fn draggable_file_tree_view_drop_signal(widget: QPtr<QWidget>) -> Signal<(*const QModelIndex, i32)> {
    unsafe {
        Signal::new(
            ::cpp_core::Ref::from_raw(widget.as_raw_ptr()).expect("attempted to construct a null Ref"),
            ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                b"2itemDrop(QModelIndex const &,int)\0",
            ),
        )
    }
}

// This function allow us to create a model compatible with draggable items
extern "C" { fn new_packed_file_model() -> *mut QStandardItemModel; }
pub fn new_packed_file_model_safe() -> QBox<QStandardItemModel> {
    unsafe { QBox::from_raw(new_packed_file_model()) }
}*/

// This function allow us to create a custom window.
extern "C" { fn launcher_window() -> *mut QMainWindow; }
pub fn launcher_window_safe() -> QBox<QMainWindow> {
    unsafe { QBox::from_raw(launcher_window()) }
}

pub fn main_window_drop_pack_signal(widget: QPtr<QWidget>) -> Signal<(*const ::qt_core::QStringList,)> {
    unsafe {
        Signal::new(
            ::cpp_core::Ref::from_raw(widget.as_raw_ptr()).expect("attempted to construct a null Ref"),
            ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                b"2openPack(QStringList const &)\0",
            ),
        )
    }
}
