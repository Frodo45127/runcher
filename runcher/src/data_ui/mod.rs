//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QTabWidget;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QTimer;

use cpp_core::CppBox;
use cpp_core::CppDeletable;

use anyhow::{anyhow, Result};
use getset::*;
use rayon::prelude::*;

use std::path::Path;
use std::rc::Rc;

use rpfm_ui_common::locale::*;
use rpfm_ui_common::utils::*;

use rpfm_lib::games::pfh_file_type::PFHFileType;
use rpfm_lib::files::{FileType, RFile, pack::Pack};
use rpfm_lib::games::GameInfo;

use crate::ffi::*;
use crate::mod_manager::{game_config::GameConfig, load_order::LoadOrder};

use self::pack_tree::*;
use self::slots::DataListUISlots;

pub mod pack_tree;
mod slots;

const VIEW_DEBUG: &str = "ui_templates/filterable_reloadable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_reloadable_tree_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct DataListUI {
    tree_view: QPtr<QTreeView>,
    model: QPtr<QStandardItemModel>,
    filter: QBox<QSortFilterProxyModel>,
    filter_line_edit: QPtr<QLineEdit>,
    filter_case_sensitive_button: QPtr<QToolButton>,
    filter_timer: QBox<QTimer>,
    reload_button: QPtr<QToolButton>,
}

#[derive(Clone, Debug, Default, Getters)]
#[getset(get = "pub")]
pub struct RFileInfo {
    path: String,
    container_name: Option<String>,
    file_type: FileType,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl DataListUI {

    pub unsafe fn new(parent: &QBox<QTabWidget>) -> Result<Rc<Self>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        let tree_view_placeholder: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let tree_view = new_pack_list_tree_view_safe(main_widget.static_upcast());
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;
        let reload_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "reload_button")?;
        reload_button.set_tool_tip(&qtr("reload_data_view"));

        // Replace the placeholder widget.
        let main_layout: QPtr<QGridLayout> = main_widget.layout().static_downcast();
        main_layout.replace_widget_2a(&tree_view_placeholder, &tree_view);
        tree_view_placeholder.delete();

        let model = new_pack_list_model_safe(tree_view.static_upcast());
        let filter = pack_list_filter_safe(main_widget.static_upcast());
        filter.set_source_model(&model);
        model.set_parent(&tree_view);
        tree_view.set_model(&filter);

        let filter_timer = QTimer::new_1a(&main_widget);
        filter_timer.set_single_shot(true);

        parent.add_tab_2a(&main_widget, &qtr("data_list_title"));

        let list = Rc::new(Self {
            tree_view,
            model,
            filter,
            filter_line_edit,
            filter_case_sensitive_button,
            filter_timer,
            reload_button,
        });

        list.set_enabled(false);

        let slots = DataListUISlots::new(&list);
        list.set_connections(&slots);

        Ok(list)
    }

    pub unsafe fn set_connections(&self, slots: &DataListUISlots) {
        self.filter_line_edit().text_changed().connect(slots.filter_line_edit());
        self.filter_case_sensitive_button().toggled().connect(slots.filter_case_sensitive_button());
        self.filter_timer().timeout().connect(slots.filter_trigger());
    }

    pub unsafe fn set_enabled(&self, enable: bool) {
        self.tree_view().set_enabled(enable);
        self.filter_line_edit().set_enabled(enable);
        self.filter_case_sensitive_button().set_enabled(enable);
    }

    pub fn generate_data(&self, game_config: &GameConfig, game: &GameInfo, game_path: &Path, load_order: &LoadOrder) -> Result<Pack> {

        // Only load this if the game path is actually a path.
        if game_path.exists() && game_path.is_dir() {

            // Build the full pack list with the vanilla packs.
            let vanilla_paths = game.ca_packs_paths(game_path)?;
            let movie_paths = load_order.movies().iter()
                .filter_map(|mod_id| game_config.mods().get(mod_id))
                .filter_map(|modd| modd.paths().first())
                .cloned()
                .collect::<Vec<_>>();

            let mut base_packs = vanilla_paths.iter().chain(movie_paths.iter())
                .filter_map(|path| Pack::read_and_merge(&[path.to_path_buf()], game, true, false, false).ok())
                .collect::<Vec<_>>();

            base_packs.sort_by(|pack_a, pack_b| if pack_a.pfh_file_type() != pack_b.pfh_file_type() {
                pack_a.pfh_file_type().cmp(&pack_b.pfh_file_type())
            } else {
                pack_a.disk_file_path().cmp(pack_b.disk_file_path())
            });

            // Generate the "merged pack" from the load order mods, and inject them into the full pack list.
            let mut mod_packs_sorted = load_order.mods().iter()
                .filter_map(|mod_id| load_order.packs().get(mod_id))
                .cloned()
                .collect::<Vec<_>>();

            // If we have movie packs in the base ones, insert the mods before the movie packs.
            //
            // If not, insert them at the end of the list.
            if let Some(pos) = base_packs.iter().position(|x| x.pfh_file_type() == PFHFileType::Movie) {
                let mut movie_packs = base_packs.split_off(pos);
                base_packs.append(&mut mod_packs_sorted);
                base_packs.append(&mut movie_packs);
            } else {
                base_packs.append(&mut mod_packs_sorted);
            };

            let full_pack = Pack::merge(&base_packs)?;

            Ok(full_pack)
        } else {
            Err(anyhow!("Game Path not found."))
        }
     }

    pub unsafe fn load(&self, game_config: &GameConfig, game: &GameInfo, game_path: &Path, load_order: &LoadOrder) -> Result<()> {
        self.tree_view.update_treeview(true, &mut TreeViewOperation::Clear);

        self.setup_columns();

        // Only load this if the game path is actually a path.
        if game_path.exists() && game_path.is_dir() {
            self.set_enabled(true);
            let full_pack = self.generate_data(game_config, game, game_path, load_order)?;

            // Then, build the tree.
            let build_data = full_pack.files().par_iter().map(|(_, file)| From::from(file)).collect();
            self.tree_view.update_treeview(true, &mut TreeViewOperation::Build(build_data));

            // Enlarge the first column if it's too small, and autoexpand the first node.
            if self.tree_view().column_width(0) < 300 {
                self.tree_view().set_column_width(0, 300);
            }

            self.tree_view().expand_to_depth(0);
        } else {
            self.set_enabled(false);
        }

        Ok(())
    }

    /// This returns the selection REVERSED, FROM BOTTOM TO TOP.
    pub unsafe fn data_list_selection(&self) -> Vec<CppBox<QModelIndex>> {
        let indexes_visual = self.tree_view().selection_model().selection().indexes();
        let mut indexes_visual = (0..indexes_visual.count_0a())
            .map(|x| indexes_visual.at(x))
            .collect::<Vec<_>>();

        // Manually sort the selection, because if the user selects with ctrl from bottom to top, this breaks hard.
        indexes_visual.sort_by_key(|index| index.row());
        indexes_visual.reverse();

        indexes_visual.iter().map(|x| self.filter().map_to_source(*x)).collect::<Vec<_>>()
    }

    pub unsafe fn setup_columns(&self) {
        self.model.set_column_count(2);

        let item_file_name = QStandardItem::from_q_string(&qtr("file_name"));
        let item_pack_name = QStandardItem::from_q_string(&qtr("pack_name"));

        self.model.set_horizontal_header_item(0, item_file_name.into_ptr());
        self.model.set_horizontal_header_item(1, item_pack_name.into_ptr());

        self.tree_view.header().set_minimum_section_size(24 * 4);
    }

    pub unsafe fn filter_list(&self) {

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&self.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = self.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        pack_list_trigger_filter_safe(self.filter(), &pattern.as_ptr());
    }

    pub unsafe fn delayed_updates(&self) {
        self.filter_timer.set_interval(500);
        self.filter_timer.start_0a();
    }
}

impl From<&RFile> for RFileInfo {
    fn from(rfile: &RFile) -> Self {
        Self {
            path: rfile.path_in_container_raw().to_owned(),
            container_name: rfile.container_name().clone(),
            file_type: rfile.file_type(),
        }
    }
}
