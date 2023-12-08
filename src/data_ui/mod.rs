//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QTimer;

use cpp_core::CppDeletable;

use anyhow::Result;
use getset::*;
use rayon::prelude::*;

use std::path::Path;
use std::rc::Rc;

use rpfm_ui_common::locale::*;
use rpfm_ui_common::utils::*;

use rpfm_lib::games::{pfh_file_type::PFHFileType, pfh_version::PFHVersion};
use rpfm_lib::files::{ContainerPath, FileType, RFile, pack::{Pack, PFHFlags}};
use rpfm_lib::games::GameInfo;

use crate::ffi::*;
use crate::mod_manager::{game_config::GameConfig, load_order::LoadOrder};

use self::pack_tree::*;
use self::slots::DataListUISlots;

mod pack_tree;
mod slots;

const VIEW_DEBUG: &str = "ui_templates/filterable_tree_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_widget.ui";

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
}

#[derive(Clone, Debug, Default, Getters)]
#[getset(get = "pub")]
pub struct ContainerInfo {
    file_name: String,
    file_path: String,
    pfh_version: PFHVersion,
    pfh_file_type: PFHFileType,
    bitmask: PFHFlags,
    compress: bool,
    timestamp: u64,
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
        let tree_view = new_mod_list_tree_view_safe(main_widget.static_upcast());
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        // Replace the placeholder widget.
        let main_layout: QPtr<QGridLayout> = main_widget.layout().static_downcast();
        main_layout.replace_widget_2a(&tree_view_placeholder, &tree_view);
        tree_view_placeholder.delete();

        let model = new_mod_list_model_safe(tree_view.static_upcast());
        let filter = mod_list_filter_safe(main_widget.static_upcast());
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
        });

        let slots = DataListUISlots::new(&list);
        list.set_connections(&slots);

        Ok(list)
    }

    pub unsafe fn set_connections(&self, slots: &DataListUISlots) {
        self.filter_line_edit().text_changed().connect(slots.filter_line_edit());
        self.filter_case_sensitive_button().toggled().connect(slots.filter_case_sensitive_button());
        self.filter_timer().timeout().connect(slots.filter_trigger());
    }

    pub unsafe fn load(&self, game_config: &GameConfig, game: &GameInfo, game_path: &Path, load_order: &LoadOrder) -> Result<()> {
        self.tree_view.update_treeview(true, TreeViewOperation::Clear);

        self.setup_columns();

        // Only load this if the game path is actually a path.
        if game_path.exists() && game_path.is_dir() {

            // Build the full pack list with the vanilla packs.
            let vanilla_paths = game.ca_packs_paths(game_path)?;
            let movie_paths = load_order.movies().iter()
                .filter_map(|mod_id| game_config.mods().get(mod_id))
                .filter_map(|modd| modd.paths().get(0))
                .cloned()
                .collect::<Vec<_>>();

            let mut base_packs = vanilla_paths.iter().chain(movie_paths.iter())
                .filter_map(|path| Pack::read_and_merge(&[path.to_path_buf()], true, false).ok())
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

            // Then, build the tree.
            let mut build_data = BuildData::new();
            build_data.data = Some((ContainerInfo::default(), full_pack.files().par_iter().map(|(_, file)| From::from(file)).collect()));
            self.tree_view.update_treeview(true, TreeViewOperation::Build(build_data));
        }

        Ok(())
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
        mod_list_trigger_filter_safe(self.filter(), &pattern.as_ptr());
    }

    pub unsafe fn delayed_updates(&self) {
        self.filter_timer.set_interval(500);
        self.filter_timer.start_0a();
    }
}

impl From<&Pack> for ContainerInfo {
    fn from(pack: &Pack) -> Self {

        // If we have no disk file for the pack, it's a new one.
        let file_name = if pack.disk_file_path().is_empty() {
            "new_file.pack"
        } else {
            pack.disk_file_path().split('/').last().unwrap_or("unknown.pack")
        };

        Self {
            file_name: file_name.to_string(),
            file_path: pack.disk_file_path().to_string(),
            pfh_version: *pack.header().pfh_version(),
            pfh_file_type: *pack.header().pfh_file_type(),
            bitmask: *pack.header().bitmask(),
            timestamp: *pack.header().internal_timestamp(),
            compress: *pack.compress(),
        }
    }
}

impl From<&RFileInfo> for ContainerInfo {
    fn from(file_info: &RFileInfo) -> Self {
        Self {
            file_name: ContainerPath::File(file_info.path().to_owned()).name().unwrap_or("unknown").to_string(),
            file_path: file_info.path().to_owned(),
            ..Default::default()
        }
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
