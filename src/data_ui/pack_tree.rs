//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QTreeView;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;
use qt_gui::QListOfQStandardItem;

use qt_core::QModelIndex;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::QPtr;

use cpp_core::CppBox;
use cpp_core::Ptr;
use cpp_core::Ref;
use cpp_core::CastFrom;

use rayon::prelude::*;

use std::cmp::Ordering;

use rpfm_lib::files::{ContainerPath, FileType};

use rpfm_ui_common::locale::qtr;

use crate::TREEVIEW_ICONS;
use super::*;

/// This const is the key of the QVariant that holds the type of each StandardItem in a `TreeView`.
const ITEM_TYPE: i32 = 20;

/// This const is used to identify an item as a PackedFile.
const ITEM_TYPE_FILE: i32 = 1;

/// This const is used to identify an item as a folder.
const ITEM_TYPE_FOLDER: i32 = 2;

/// This const is used to identify an item as a PackFile.
const ITEM_TYPE_PACKFILE: i32 = 3;

//-------------------------------------------------------------------------------//
//                          Enums & Structs (and trait)
//-------------------------------------------------------------------------------//

/// This trait adds multiple util functions to the `TreeView` you implement it for.
///
/// Keep in mind that this trait has been created with `PackFile TreeView's` in mind, so his methods
/// may not be suitable for all purposes.
pub trait PackTree {

    /// This function is used to expand the entire path from the PackFile to an specific item in the `TreeView`.
    ///
    /// It returns the `ModelIndex` of the final item of the path, or None if it wasn't found or it's hidden by the filter.
    unsafe fn expand_treeview_to_item(&self, path: &str) -> Option<Ptr<QModelIndex>>;

    /// This function is used to expand an item and all it's children recursively.
    unsafe fn expand_all_from_item(tree_view: &QTreeView, item: Ptr<QStandardItem>, first_item: bool);

    /// This function is used to expand an item and all it's children recursively.
    unsafe fn expand_all_from_type(tree_view: &QTreeView, item: &ContainerPath);

    /// This function gives you the items selected in the provided `TreeView`.
    unsafe fn get_items_from_selection(&self, has_filter: bool) -> Vec<Ptr<QStandardItem>>;

    /// This function gives you the `TreeViewTypes` of the items selected in the provided TreeView.
    unsafe fn get_item_types_from_selection(&self, has_filter: bool) -> Vec<ContainerPath>;

    /// This function returns the `ContainerPath`s not hidden by the applied filter corresponding to the current selection.
    ///
    /// This always assumes the `TreeView` has a filter. It'll die horrendously otherwise.
    unsafe fn get_item_types_from_selection_filtered(&self) -> Vec<ContainerPath>;

    /// This function gives you the item corresponding to an specific `ContainerPath`.
    unsafe fn item_from_path(path: &ContainerPath, model: &QPtr<QStandardItemModel>) -> Ptr<QStandardItem>;

    /// This function returns the `ContainerPath` of the provided item. Unsafe version.
    unsafe fn get_type_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> ContainerPath;

    /// This function is used to get the path of a specific Item in a StandardItemModel. Unsafe version.
    unsafe fn get_path_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> String;

    /// This function is used to get the path of a specific ModelIndex in a StandardItemModel. Unsafe version.
    unsafe fn get_path_from_index(index: Ref<QModelIndex>, model: &QPtr<QStandardItemModel>) -> String;

    /// This function gives you the path of the items selected in the provided TreeView.
    unsafe fn get_path_from_selection(&self) -> Vec<String>;

    /// This function returns the currently visible children of the given parent, and adds them as `ContainerPath`s to the provided list.
    unsafe fn visible_children_of_item(&self, parent: &QStandardItem, visible_paths: &mut Vec<ContainerPath>);

    /// This function takes care of EVERY operation that manipulates the provided TreeView.
    /// It does one thing or another, depending on the operation we provide it.
    ///
    /// BIG NOTE: Each StandardItem should keep track of his own status, meaning that their data means:
    /// - Position 20: Type. 1 is File, 2 is Folder, 4 is PackFile.
    /// - Position 21: Status. 0 is untouched, 1 is added, 2 is modified.
    /// In case you don't realise, those are bitmasks.
    unsafe fn update_treeview(&self, has_filter: bool, operation: &mut TreeViewOperation);
}

/// This enum has the different possible operations we can do in a `TreeView`.
#[derive(Clone, Debug)]
pub enum TreeViewOperation {
    Build(Vec<RFileInfo>),
    Clear,
}

//-------------------------------------------------------------------------------//
//                      Implementations of `PackTree`
//-------------------------------------------------------------------------------//

impl PackTree for QPtr<QTreeView> {

    unsafe fn expand_treeview_to_item(&self, path: &str) -> Option<Ptr<QModelIndex>> {
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        // Get the first item's index, as that one should always exist (the Packfile).
        let root_item = model.item_1a(0);
        if !root_item.is_null() {

            let mut item = root_item;
            let model_index = model.index_2a(0, 0);
            let filtered_index = filter.map_from_source(&model_index);

            // If it's valid (filter didn't hid it away), we expand it and search among its children the next one to expand.
            if filtered_index.is_valid() {
                if !self.is_expanded(&filtered_index) {
                    self.expand(&filtered_index);
                }

                // Indexes to see how deep we must go.
                let mut index = 0;
                let path = path.split('/').collect::<Vec<_>>();
                let path_deep = path.len();
                if path_deep > 0 {

                    loop {

                        let mut not_found = true;
                        for row in 0..item.row_count() {
                            let child = item.child_1a(row);

                            // In the last cycle, we're interested in files, not folders.
                            if index == (path_deep -1) {

                                if child.has_children() { continue; }

                                // We guarantee that the name of the files/folders is unique, so we use it to find the one to expand.
                                if path[index] == child.text().to_std_string() {
                                    item = child;

                                    let model_index = model.index_from_item(item);
                                    let filtered_index = filter.map_from_source(&model_index);

                                    if filtered_index.is_valid() { return Some(filtered_index.into_ptr()); }
                                    else { return None }
                                }
                            }

                            // In the rest, we look for children with children of its own.
                            else {
                                if !child.has_children() { continue; }

                                // We guarantee that the name of the files/folders is unique, so we use it to find the one to expand.
                                if path[index] == child.text().to_std_string() {
                                    item = child;
                                    index += 1;
                                    not_found = false;

                                    // Expand the folder, if exists.
                                    let model_index = model.index_from_item(item);
                                    let filtered_index = filter.map_from_source(&model_index);

                                    if filtered_index.is_valid() { self.expand(&filtered_index); }
                                    else { not_found = true; }

                                    // Break the loop.
                                    break;
                                }
                            }
                        }

                        // If the child was not found, stop and return the parent.
                        if not_found { break; }
                    }
                }
            }
        }
        None
    }

    unsafe fn expand_all_from_type(tree_view: &QTreeView, item: &ContainerPath) {
        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let item = Self::item_from_path(item, &model);
        Self::expand_all_from_item(tree_view, item, true);
    }

    unsafe fn expand_all_from_item(tree_view: &QTreeView, item: Ptr<QStandardItem>, first_item: bool) {
        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        // First, expand our item, then expand its children.
        let model_index = model.index_from_item(item);
        if first_item {
            let filtered_index = filter.map_from_source(&model_index);
            if filtered_index.is_valid() {
                tree_view.expand(&filtered_index);
            }
        }
        for row in 0..item.row_count() {
            let child = item.child_1a(row);
            if child.has_children() {
                let model_index = model.index_from_item(item);
                let filtered_index = filter.map_from_source(&model_index);
                if filtered_index.is_valid() {
                    tree_view.expand(&filtered_index);
                    Self::expand_all_from_item(tree_view, Ptr::cast_from(child), false);
                }
            }
        }
    }

    unsafe fn get_items_from_selection(&self, has_filter: bool) -> Vec<Ptr<QStandardItem>> {
        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };
        let model: QPtr<QStandardItemModel> = if let Some(ref filter) = filter { filter.source_model().static_downcast() } else { self.model().static_downcast()};

        let indexes_visual = self.selection_model().selection().indexes();
        let mut indexes_visual = (0..indexes_visual.count_0a()).rev().map(|x| indexes_visual.take_at(x)).collect::<Vec<CppBox<QModelIndex>>>();
        indexes_visual.reverse();
        let indexes_real = if let Some(filter) = filter {
            indexes_visual.iter().map(|x| filter.map_to_source(x.as_ref())).collect::<Vec<CppBox<QModelIndex>>>()
        } else {
            indexes_visual
        };

        indexes_real.iter().map(|x| model.item_from_index(x.as_ref())).collect()
    }

    unsafe fn get_item_types_from_selection(&self, has_filter: bool) -> Vec<ContainerPath> {
        let items = self.get_items_from_selection(has_filter);

        let model: QPtr<QStandardItemModel> = if has_filter {
            let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
            filter.source_model().static_downcast()
        } else {
            self.model().static_downcast()
        };

        items.iter().map(|x| Self::get_type_from_item(*x, &model)).collect()
    }

    unsafe fn get_item_types_from_selection_filtered(&self)-> Vec<ContainerPath> {
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        let mut item_types = vec![];
        let item_types_selected = self.get_item_types_from_selection(true);
        for item_type in &item_types_selected {
            match item_type {
                 ContainerPath::File(_) => item_types.push(item_type.clone()),
                 ContainerPath::Folder(_) => {
                    item_types.push(item_type.clone());
                    let item = Self::item_from_path(item_type, &model);
                    self.visible_children_of_item(&item, &mut item_types);
                 }
            }
        }

        item_types = ContainerPath::dedup(&item_types);
        item_types
    }

    unsafe fn visible_children_of_item(&self, parent: &QStandardItem, visible_paths: &mut Vec<ContainerPath>) {
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        for row in 0..parent.row_count() {
            let child = parent.child_1a(row);
            let child_index = child.index();
            let filtered_index = filter.map_from_source(&child_index);
            if filtered_index.is_valid() {
                if child.has_children() {
                    self.visible_children_of_item(&child, visible_paths);
                }
                else {
                    visible_paths.push(Self::get_type_from_item(child, &model));
                }
            }
        }
    }

    unsafe fn item_from_path(path: &ContainerPath, model: &QPtr<QStandardItemModel>) -> Ptr<QStandardItem> {
        let mut item = model.item_1a(0);
        let is_file = path.is_file();
        let path = path.path_raw();
        let count = path.split('/').count() - 1;

        for (index, path_element) in path.split('/').enumerate() {
            let children_count = item.row_count();

            // If we reached the folder of the item...
            if index == count {
                let path_element_q_string = QString::from_std_str(path_element);
                for row in 0..children_count {
                    let child = item.child_1a(row);

                    // We ignore files or folders, depending on what we want to create.
                    if is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FOLDER { continue }
                    if !is_file && child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                    let compare = child.text().compare_q_string(&path_element_q_string);
                    match compare.cmp(&0) {
                        Ordering::Equal => {
                            item = child;
                            break;
                        },

                        // If it's less, we still can find the item.
                        Ordering::Less => {}

                        // If it's greater, we passed the item. In theory, this can't happen.
                        Ordering::Greater => {
                            dbg!(child.text().to_std_string());
                            dbg!(path_element_q_string.to_std_string());
                            dbg!("bug?");
                            break;
                        },
                    }
                }
                break;
            }

            // If we are not still in the folder of the file...
            else {

                // Get the amount of children of the current item and go through them until we find our folder.
                let mut not_found = true;
                let text_to_search = QString::from_std_str(path_element);
                for row in 0..children_count {
                    let child = item.child_1a(row);

                    // Items are sorted with folders first. If we start finding files, we already skipped our item.
                    if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { break; }

                    let compare = QString::compare_2_q_string(child.text().as_ref(), text_to_search.as_ref());
                    match compare.cmp(&0) {
                        Ordering::Equal => {
                            item = child;
                            not_found = false;
                            break;
                        },

                        // If it's less, we still can find the item.
                        Ordering::Less => {}

                        // If it's greater, we passed all the possible items and we can no longer find the folder.
                        Ordering::Greater => {
                            break;
                        },
                    }
                }

                // If the child was not found, stop and return the parent.
                if not_found { break; }
            }
        }

        item
    }

    unsafe fn get_type_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> ContainerPath {
        match item.data_1a(ITEM_TYPE).to_int_0a() {
            ITEM_TYPE_FILE => ContainerPath::File(Self::get_path_from_item(item, model)),
            ITEM_TYPE_FOLDER => ContainerPath::Folder(Self::get_path_from_item(item, model)),
            ITEM_TYPE_PACKFILE => ContainerPath::Folder(String::new()),
            _ => unreachable!("from_type {}", item.data_1a(ITEM_TYPE).to_int_0a())
        }
    }

    unsafe fn get_path_from_item(item: Ptr<QStandardItem>, model: &QPtr<QStandardItemModel>) -> String {
        let index = item.index();
        Self::get_path_from_index(index.as_ref(), model)
    }

    unsafe fn get_path_from_index(index: Ref<QModelIndex>, model: &QPtr<QStandardItemModel>) -> String {

        // The logic is simple: we loop from item to parent until we reach the top.
        let mut path = vec![];
        let mut index = index;
        let mut parent;

        // Loop until we reach the root index.
        loop {
            let text = model.data_1a(index).to_string().to_std_string();
            parent = index.parent();

            // If the parent is valid, it's the new item. Otherwise, we stop without adding it (we don't want the PackFile's name in).
            if parent.is_valid() {
                path.push(text);
                index = parent.as_ref();
            } else { break; }
        }

        // Reverse it, as we want it from arent to children.
        path.reverse();
        path.join("/")
    }

    unsafe fn get_path_from_selection(&self) -> Vec<String> {

        // Create the vector to hold the Paths and get the selected indexes of the TreeView.
        let filter: QPtr<QSortFilterProxyModel> = self.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let selection_model = self.selection_model();

        let mut paths: Vec<_> = vec![];
        let indexes = filter.map_selection_to_source(&selection_model.selection()).indexes();
        for index_num in 0..indexes.count_0a() {
            paths.push(Self::get_path_from_index(indexes.at(index_num), &model));
        }
        paths
    }

    unsafe fn update_treeview(&self, has_filter: bool, operation: &mut TreeViewOperation) {
        let filter: Option<QPtr<QSortFilterProxyModel>> = if has_filter { Some(self.model().static_downcast()) } else { None };
        let model: QPtr<QStandardItemModel> = if let Some(ref filter) = filter { filter.source_model().static_downcast() } else { self.model().static_downcast() };

        // Make sure we don't try to update the view until the model is done.
        self.set_updates_enabled(false);
        self.selection_model().block_signals(true);

        // We act depending on the operation requested.
        match operation {
            TreeViewOperation::Build(ref mut packed_files_data) => {
                if packed_files_data.is_empty() {
                    self.set_updates_enabled(true);
                    self.selection_model().block_signals(false);
                    return
                }

                let big_parent = QStandardItem::from_q_string(&qtr("game_data"));
                big_parent.set_editable(false);
                big_parent.set_data_2a(&QVariant::from_int(ITEM_TYPE_PACKFILE), ITEM_TYPE);

                TREEVIEW_ICONS.set_standard_item_icon(&big_parent, Some(&FileType::Pack));
                let big_parent = big_parent.into_ptr();

                // We sort the paths with this horrific monster I don't want to touch ever again, using the following format:
                // - FolderA
                // - FolderB
                // - FileA
                // - FileB
                sort_folders_before_files_alphabetically_file_infos(packed_files_data);

                // Optimisation: prebuilt certain file-related data before entering the TreeView build loop. This improves performances by about 5%.
                let packed_files_data = packed_files_data.par_iter().map(|data| (data.path().split('/').count() - 1, data.path().split('/'), data)).collect::<Vec<_>>();

                let variant_type_file = QVariant::from_int(ITEM_TYPE_FILE);
                let variant_type_folder = QVariant::from_int(ITEM_TYPE_FOLDER);

                let base_file_item = QStandardItem::from_q_string(&QString::new());
                base_file_item.set_editable(false);
                base_file_item.set_data_2a(&variant_type_file, ITEM_TYPE);
                let base_file_item = atomic_from_cpp_box(base_file_item);

                let base_folder_item = QStandardItem::from_q_string(&QString::new());
                base_folder_item.set_editable(false);
                base_folder_item.set_data_2a(&variant_type_folder, ITEM_TYPE);
                TREEVIEW_ICONS.set_standard_item_icon(&base_folder_item, None);

                // Optimisation: Premade the file items before building the tree. This gives us around 20% better times when building WH3 depedencies TreeView.
                let mut files = packed_files_data.par_iter().rev().map(|(_,_,file_info)| {
                    let file = (*ref_from_atomic(&base_file_item)).clone();
                    let pack = (*ref_from_atomic(&base_file_item)).clone();

                    if let Some((_, name)) = file_info.path().rsplit_once('/') {
                        file.set_text(&QString::from_std_str(name));
                    } else {
                        file.set_text(&QString::from_std_str(file_info.path()));
                    }

                    if let Some(container_name) = file_info.container_name() {
                        pack.set_text(&QString::from_std_str(container_name));
                    }

                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(file_info.file_type()));

                    let row = QListOfQStandardItem::new();
                    row.append_q_standard_item(&file.as_mut_raw_ptr());
                    row.append_q_standard_item(&pack.as_mut_raw_ptr());

                    atomic_from_ptr(row.into_ptr())
                }).collect::<Vec<_>>();

                // Once we get the entire path list sorted, we add the paths to the model one by one,
                // skipping duplicate entries.
                for (count, path_split, _) in packed_files_data {

                    // First, we reset the parent to the big_parent (the PackFile).
                    // Then, we form the path ("parent -> child" style path) to add to the model.
                    let mut parent = big_parent;

                    for (index_in_path, name) in path_split.enumerate() {
                        let name = QString::from_std_str(name);

                        // If it's the last string in the file path, it's a file, so we add it to the model.
                        if index_in_path == count {
                            parent.append_row_q_list_of_q_standard_item(ref_from_atomic(&files.pop().unwrap()));
                        }

                        // If it's a folder, we check first if it's already in the TreeView using the following
                        // logic:
                        // - If the current parent has a child, it should be a folder already in the TreeView,
                        //   so we check all his children.
                        // - If any of them is equal to the current folder we are trying to add and it has at
                        //   least one child, it's a folder exactly like the one we are trying to add, so that
                        //   one becomes our new parent.
                        // - If there is no equal folder to the one we are trying to add, we add it, turn it
                        //   into the new parent, and repeat.
                        else {

                            // If the current parent has at least one child, check if the folder already exists.
                            let mut duplicate_found = false;
                            let children_len = parent.row_count();

                            if parent.has_children() {

                                // It's a folder, so we check his children. We are only interested in
                                // folders, so ignore the files. Reverse because due to the sorting it's almost
                                // sure the last folder is the one we want.
                                for index in (0..children_len).rev() {
                                    let child = parent.child_2a(index, 0);
                                    if child.data_1a(ITEM_TYPE).to_int_0a() == ITEM_TYPE_FILE { continue }

                                    // Get his text. If it's the same folder we are trying to add, this is our parent now.
                                    let compare = child.text().compare_q_string(&name);
                                    match compare.cmp(&0) {
                                        Ordering::Equal => {
                                            parent = parent.child_1a(index);
                                            duplicate_found = true;
                                            break;
                                        },

                                        // Optimization: We get the paths pre-sorted. If the last folder cannot be under our folder, stop iterating.
                                        Ordering::Less => {
                                            break;
                                        },
                                        Ordering::Greater => {},
                                    }
                                }
                            }

                            // If our current parent doesn't have anything, just add it as a new folder.
                            if !duplicate_found {
                                let folder = base_folder_item.clone();
                                let packs = (*ref_from_atomic(&base_file_item)).clone();

                                folder.set_text(&name);

                                let row = QListOfQStandardItem::new();
                                row.append_q_standard_item(&folder.as_mut_raw_ptr());
                                row.append_q_standard_item(&packs.as_mut_raw_ptr());

                                parent.append_row_q_list_of_q_standard_item(&row);

                                // This is our parent now.
                                parent = parent.child_1a(children_len);
                            }
                        }
                    }
                }

                // Delay adding the big parent as much as we can, as otherwise the signals triggered when adding a file can slow this down to a crawl.
                model.append_row_q_standard_item(big_parent);
            },

            // If we want to remove everything from the TreeView...
            TreeViewOperation::Clear => model.clear(),
        }

        // Re-enable the view.
        self.set_updates_enabled(true);
        self.selection_model().block_signals(false);
    }
}

//----------------------------------------------------------------//
// Helpers to control the main TreeView.
//----------------------------------------------------------------//

// We sort the paths with this horrific monster I don't want to touch ever again, using the following format:
// - FolderA
// - FolderB
// - FileA
// - FileB
fn sort_folders_before_files_alphabetically_file_infos(files: &mut Vec<RFileInfo>) {
    files.par_sort_unstable_by(|a, b| {
        let a_path = a.path();
        let b_path = b.path();

        sort_folders_before_files_alphabetically_paths(a_path, b_path)
    });
}

fn sort_folders_before_files_alphabetically_paths(a_path: &str, b_path: &str) -> Ordering {
    let mut a_iter = a_path.rmatch_indices('/');
    let mut b_iter = b_path.rmatch_indices('/');

    let (a_last_split, a_len) = {
        match a_iter.next() {
            Some((index, _)) => (index, a_iter.count() + 1),
            None => (0, 0),
        }
    };
    let (b_last_split, b_len) = {
        match b_iter.next() {
            Some((index, _)) => (index, b_iter.count() + 1),
            None => (0, 0),
        }
    };

    // Short-circuit cases: one or both files on root.
    if a_last_split == 0 && b_last_split == 0 {
        return a_path.cmp(b_path);
    } else if a_last_split == 0 {
        return Ordering::Greater;
    } else if b_last_split == 0 {
        return Ordering::Less;
    }

    // Short-circuit: both are files under the same amount of subfolders.
    if a_len == b_len {
        a_path.cmp(b_path)
    } else if a_len > b_len {
        if a_path.starts_with(&b_path[..=b_last_split]) {
            Ordering::Less
        } else {
            a_path.cmp(b_path)
        }
    } else if b_path.starts_with(&a_path[..=a_last_split]) {
        Ordering::Greater
    } else {
        a_path.cmp(b_path)
    }
}
