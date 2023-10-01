//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QAction;
use qt_widgets::QActionGroup;
use qt_widgets::QApplication;
use qt_widgets::QToolBar;
use qt_widgets::{QDialog, QDialogButtonBox, q_dialog_button_box::StandardButton};
use qt_widgets::QLabel;
use qt_widgets::QMainWindow;
use qt_widgets::QSplitter;
use qt_widgets::QTextEdit;
use qt_widgets::{QMessageBox, q_message_box};
use qt_widgets::QWidget;

use qt_gui::QIcon;
use qt_gui::QStandardItem;

use qt_core::CheckState;
use qt_core::Orientation;
use qt_core::QBox;
use qt_core::QCoreApplication;
use qt_core::QFlags;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QSize;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::SlotNoArgs;

use cpp_core::CppBox;

use anyhow::{anyhow, Result};
use getset::Getters;
use rayon::prelude::*;
use sha256::try_digest;

use std::cmp::Reverse;
use std::collections::HashMap;
use std::env::{args, current_exe};
use std::fs::{copy, DirBuilder, File};
use std::io::{BufWriter, Read, Write};
#[cfg(target_os = "windows")] use std::os::windows::{prelude::MetadataExt, process::CommandExt};
use std::path::{Path, PathBuf};
use std::process::{Command as SystemCommand, exit};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::UNIX_EPOCH;

use rpfm_lib::binary::WriteBytes;
use rpfm_lib::files::{EncodeableExtraData, esf::NodeType, pack::Pack, RFile, RFileDecoded};
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType, supported_games::*};
use rpfm_lib::integrations::{git::*, log::*};
use rpfm_lib::schema::Schema;
use rpfm_lib::utils::files_from_subdir;

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::clone;
use rpfm_ui_common::locale::*;
use rpfm_ui_common::PROGRAM_PATH;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::actions_ui::ActionsUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::DARK_PALETTE;
use crate::ffi::launcher_window_safe;
use crate::games::*;
use crate::mod_manager::{game_config::GameConfig, mods::{Mod, ShareableMod}, profiles::Profile, saves::Save, integrations::*};
use crate::LIGHT_PALETTE;
use crate::LIGHT_STYLE_SHEET;
use crate::mod_list_ui::*;
use crate::pack_list_ui::PackListUI;
use crate::SCHEMA;
use crate::settings_ui::*;
use crate::SUPPORTED_GAMES;
use crate::updater::*;

use self::slots::AppUISlots;

pub mod slots;

#[cfg(target_os = "windows")] const CREATE_NO_WINDOW: u32 = 0x08000000;
//const DETACHED_PROCESS: u32 = 0x00000008;

const LOAD_ORDER_STRING_VIEW_DEBUG: &str = "ui_templates/load_order_string_dialog.ui";
const LOAD_ORDER_STRING_VIEW_RELEASE: &str = "ui/load_order_string_dialog.ui";

const RESERVED_PACK_NAME: &str = "zzzzzzzzzzzzzzzzzzzzrun_you_fool_thron.pack";
const RESERVED_PACK_NAME_ALTERNATIVE: &str = "!!!!!!!!!!!!!!!!!!!!!run_you_fool_thron.pack";
const MERGE_ALL_PACKS_PACK_NAME: &str = "merge_me_sideways_honey";

#[allow(dead_code)] const VANILLA_MOD_LIST_FILE_NAME: &str = "used_mods.txt";
#[allow(dead_code)] const CUSTOM_MOD_LIST_FILE_NAME: &str = "mod_list.txt";
#[allow(dead_code)] const USER_SCRIPT_FILE_NAME: &str = "user.script.txt";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access to all the static widgets/actions created at the start of the program.
///
/// This means every widget/action that's static and created on start (menus, window,...) should be here.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct AppUI {

    //-------------------------------------------------------------------------------//
    // Main Window.
    //-------------------------------------------------------------------------------//
    main_window: QBox<QMainWindow>,

    //-------------------------------------------------------------------------------//
    // `Game Selected` menu.
    //-------------------------------------------------------------------------------//
    game_selected_pharaoh: QPtr<QAction>,
    game_selected_warhammer_3: QPtr<QAction>,
    game_selected_troy: QPtr<QAction>,
    game_selected_three_kingdoms: QPtr<QAction>,
    game_selected_warhammer_2: QPtr<QAction>,
    game_selected_warhammer: QPtr<QAction>,
    game_selected_thrones_of_britannia: QPtr<QAction>,
    game_selected_attila: QPtr<QAction>,
    game_selected_rome_2: QPtr<QAction>,
    game_selected_shogun_2: QPtr<QAction>,
    game_selected_napoleon: QPtr<QAction>,
    game_selected_empire: QPtr<QAction>,

    game_selected_group: QBox<QActionGroup>,

    //-------------------------------------------------------------------------------//
    // `About` menu.
    //-------------------------------------------------------------------------------//
    about_about_qt: QPtr<QAction>,
    about_about_runcher: QPtr<QAction>,
    about_check_updates: QPtr<QAction>,
    about_check_schema_updates: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `Actions` section.
    //-------------------------------------------------------------------------------//
    actions_ui: Arc<ActionsUI>,

    //-------------------------------------------------------------------------------//
    // `Mod List` section.
    //-------------------------------------------------------------------------------//
    mod_list_ui: Arc<ModListUI>,

    //-------------------------------------------------------------------------------//
    // `Pack List` section.
    //-------------------------------------------------------------------------------//
    pack_list_ui: Arc<PackListUI>,

    //-------------------------------------------------------------------------------//
    // Extra stuff
    //-------------------------------------------------------------------------------//
    focused_widget: Rc<RwLock<Option<QPtr<QWidget>>>>,
    disabled_counter: Rc<RwLock<u32>>,

    game_config: Arc<RwLock<Option<GameConfig>>>,
    game_profiles: Arc<RwLock<HashMap<String, Profile>>>,
    game_saves: Arc<RwLock<Vec<Save>>>,

    // Game selected. Unlike RPFM, here it's not a global.
    game_selected: Rc<RwLock<GameInfo>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AppUI {

    /// This function creates an entire `AppUI` struct. Used to create the entire UI at start.
    pub unsafe fn new() -> Result<Arc<Self>> {

        // Initialize and configure the main window.
        let main_window = launcher_window_safe(setting_bool("dark_mode"));
        let central_widget = QWidget::new_1a(&main_window);
        let central_layout = create_grid_layout(central_widget.static_upcast());
        main_window.set_central_widget(&central_widget);
        main_window.resize_2a(1300, 1000);
        main_window.set_window_title(&QString::from_std_str("The Runcher"));
        QApplication::set_window_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/runcher.png", ASSETS_PATH.to_string_lossy()))));

        let splitter = QSplitter::from_q_widget(&central_widget);
        let left_widget = QWidget::new_1a(&splitter);
        let right_widget = QWidget::new_1a(&splitter);
        let _ = create_grid_layout(left_widget.static_upcast());
        let _ = create_grid_layout(right_widget.static_upcast());
        splitter.set_stretch_factor(0, 1);
        right_widget.set_minimum_width(540);

        central_layout.add_widget_5a(splitter.into_raw_ptr(), 0, 1, 1, 1);

        // Get the menu and status bars.
        let menu_bar = main_window.menu_bar();
        let status_bar = main_window.status_bar();
        status_bar.set_size_grip_enabled(false);
        let menu_bar_about = menu_bar.add_menu_q_string(&qtr("menu_bar_about"));

        //-----------------------------------------------//
        // `Game Selected` Menu.
        //-----------------------------------------------//

        // Add a game selected toolbar on the left side of the screen.
        let game_selected_bar = QToolBar::from_q_widget(&central_widget);
        let _ = create_grid_layout(game_selected_bar.static_upcast());
        game_selected_bar.set_orientation(Orientation::Vertical);
        game_selected_bar.set_icon_size(&QSize::new_2a(64, 64));
        game_selected_bar.set_fixed_width(64);

        let icon_folder = format!("{}/icons/", ASSETS_PATH.to_string_lossy());
        let game_selected_pharaoh = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_PHARAOH).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_PHARAOH));
        let game_selected_warhammer_3 = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_WARHAMMER_3));
        let game_selected_troy = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_TROY));
        let game_selected_three_kingdoms = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_THREE_KINGDOMS));
        let game_selected_warhammer_2 = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_WARHAMMER_2));
        let game_selected_warhammer = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_WARHAMMER));
        let game_selected_thrones_of_britannia = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_THRONES_OF_BRITANNIA));
        let game_selected_attila = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_ATTILA));
        let game_selected_rome_2 = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_ROME_2));
        let game_selected_shogun_2 = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_SHOGUN_2));
        let game_selected_napoleon = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_NAPOLEON));
        let game_selected_empire = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_EMPIRE));

        let game_selected_group = QActionGroup::new(&game_selected_bar);

        // Configure the `Game Selected` Menu.
        game_selected_group.add_action_q_action(&game_selected_pharaoh);
        game_selected_group.add_action_q_action(&game_selected_warhammer_3);
        game_selected_group.add_action_q_action(&game_selected_troy);
        game_selected_group.add_action_q_action(&game_selected_three_kingdoms);
        game_selected_group.add_action_q_action(&game_selected_warhammer_2);
        game_selected_group.add_action_q_action(&game_selected_warhammer);
        game_selected_group.add_action_q_action(&game_selected_thrones_of_britannia);
        game_selected_group.add_action_q_action(&game_selected_attila);
        game_selected_group.add_action_q_action(&game_selected_rome_2);
        game_selected_group.add_action_q_action(&game_selected_shogun_2);
        game_selected_group.add_action_q_action(&game_selected_napoleon);
        game_selected_group.add_action_q_action(&game_selected_empire);
        game_selected_pharaoh.set_checkable(true);
        game_selected_warhammer_3.set_checkable(true);
        game_selected_troy.set_checkable(true);
        game_selected_three_kingdoms.set_checkable(true);
        game_selected_warhammer_2.set_checkable(true);
        game_selected_warhammer.set_checkable(true);
        game_selected_thrones_of_britannia.set_checkable(true);
        game_selected_attila.set_checkable(true);
        game_selected_rome_2.set_checkable(true);
        game_selected_shogun_2.set_checkable(true);
        game_selected_napoleon.set_checkable(true);
        game_selected_empire.set_checkable(true);

        central_layout.add_widget_5a(game_selected_bar.into_raw_ptr(), 0, 0, 1, 1);

        //-----------------------------------------------//
        // `About` Menu.
        //-----------------------------------------------//
        let about_about_qt = menu_bar_about.add_action_q_string(&qtr("about_qt"));
        let about_about_runcher = menu_bar_about.add_action_q_string(&qtr("about_runcher"));
        let about_check_updates = menu_bar_about.add_action_q_string(&qtr("check_updates"));
        let about_check_schema_updates = menu_bar_about.add_action_q_string(&qtr("check_schema_updates"));

        //-------------------------------------------------------------------------------//
        // `Actions` section.
        //-------------------------------------------------------------------------------//
        let actions_ui = ActionsUI::new(&right_widget)?;

        //-------------------------------------------------------------------------------//
        // `Mod List` section.
        //-------------------------------------------------------------------------------//
        let mod_list_ui = ModListUI::new(&left_widget)?;

        //-------------------------------------------------------------------------------//
        // `Pack List` section.
        //-------------------------------------------------------------------------------//
        let pack_list_ui = PackListUI::new(&right_widget)?;

        let app_ui = Arc::new(Self {

            //-------------------------------------------------------------------------------//
            // Main Window.
            //-------------------------------------------------------------------------------//
            main_window,

            //-------------------------------------------------------------------------------//
            // "Game Selected" menu.
            //-------------------------------------------------------------------------------//
            game_selected_pharaoh,
            game_selected_warhammer_3,
            game_selected_troy,
            game_selected_three_kingdoms,
            game_selected_warhammer_2,
            game_selected_warhammer,
            game_selected_thrones_of_britannia,
            game_selected_attila,
            game_selected_rome_2,
            game_selected_shogun_2,
            game_selected_napoleon,
            game_selected_empire,

            game_selected_group,

            //-------------------------------------------------------------------------------//
            // "About" menu.
            //-------------------------------------------------------------------------------//
            about_about_qt,
            about_about_runcher,
            about_check_updates,
            about_check_schema_updates,

            //-------------------------------------------------------------------------------//
            // `Actions` section.
            //-------------------------------------------------------------------------------//
            actions_ui,

            //-------------------------------------------------------------------------------//
            // `Mod List` section.
            //-------------------------------------------------------------------------------//
            mod_list_ui,

            //-------------------------------------------------------------------------------//
            // `Pack List` section.
            //-------------------------------------------------------------------------------//
            pack_list_ui,

            //-------------------------------------------------------------------------------//
            // "Extra stuff" menu.
            //-------------------------------------------------------------------------------//
            focused_widget: Rc::new(RwLock::new(None)),
            disabled_counter: Rc::new(RwLock::new(0)),
            game_config: Arc::new(RwLock::new(None)),
            game_profiles: Arc::new(RwLock::new(HashMap::new())),
            game_saves: Arc::new(RwLock::new(vec![])),

            // NOTE: This loads arena on purpose, so ANY game selected triggers a game change properly.
            game_selected: Rc::new(RwLock::new(SUPPORTED_GAMES.game("arena").unwrap().clone())),
        });

        let slots = AppUISlots::new(&app_ui);
        app_ui.set_connections(&slots);

        // Initialize settings.
        init_settings(&app_ui.main_window().static_upcast());

        // Disable the games we don't have a path for (uninstalled) and Shogun 2, as it's not supported yet.
        for (_, game) in SUPPORTED_GAMES.games_sorted().iter().enumerate() {
            let has_exe = game.executable_path(&setting_path(game.key())).filter(|path| path.is_file()).is_some();
            match game.key() {
                KEY_PHARAOH => app_ui.game_selected_pharaoh().set_enabled(has_exe),
                KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3().set_enabled(has_exe),
                KEY_TROY => app_ui.game_selected_troy().set_enabled(has_exe),
                KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms().set_enabled(has_exe),
                KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2().set_enabled(has_exe),
                KEY_WARHAMMER => app_ui.game_selected_warhammer().set_enabled(has_exe),
                KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia().set_enabled(has_exe),
                KEY_ATTILA => app_ui.game_selected_attila().set_enabled(has_exe),
                KEY_ROME_2 => app_ui.game_selected_rome_2().set_enabled(has_exe),
                KEY_SHOGUN_2 => app_ui.game_selected_shogun_2().set_enabled(has_exe),
                KEY_NAPOLEON => app_ui.game_selected_napoleon().set_enabled(has_exe),
                KEY_EMPIRE => app_ui.game_selected_empire().set_enabled(has_exe),
                _ => {},
            }
        }

        // Load the correct theme.
        Self::reload_theme();

        // Apply last ui state.
        app_ui.main_window().restore_geometry(&setting_byte_array("geometry"));
        app_ui.main_window().restore_state_1a(&setting_byte_array("windowState"));

        // Show the Main Window.
        app_ui.main_window().show();
        app_ui.toggle_main_window(false);
        log_to_status_bar(app_ui.main_window().status_bar(), "Initializing, please wait...");

        // Trigger an event loop cycle so the window is shown, then we do the expensive stuff.
        let event_loop = qt_core::QEventLoop::new_0a();
        event_loop.process_events_0a();

        // Set the game selected based on the default game. If we passed a game through an argument, use that one.
        //
        // Note: set_checked does *NOT* trigger the slot for changing game selected. We need to trigger that one manually.
        let mut default_game = setting_string("default_game");
        let args = args().collect::<Vec<String>>();
        if args.len() == 2 {
            match &*args[1] {
                KEY_PHARAOH |
                KEY_WARHAMMER_3 |
                KEY_TROY |
                KEY_THREE_KINGDOMS |
                KEY_WARHAMMER_2 |
                KEY_WARHAMMER |
                KEY_THRONES_OF_BRITANNIA |
                KEY_ATTILA |
                KEY_ROME_2 |
                KEY_SHOGUN_2 |
                KEY_NAPOLEON |
                KEY_EMPIRE => default_game = args[1].to_string(),
                _ => {},
            }
        }

        match &*default_game {
            KEY_PHARAOH => app_ui.game_selected_pharaoh().set_checked(true),
            KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3().set_checked(true),
            KEY_TROY => app_ui.game_selected_troy().set_checked(true),
            KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms().set_checked(true),
            KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2().set_checked(true),
            KEY_WARHAMMER => app_ui.game_selected_warhammer().set_checked(true),
            KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia().set_checked(true),
            KEY_ATTILA => app_ui.game_selected_attila().set_checked(true),
            KEY_ROME_2 => app_ui.game_selected_rome_2().set_checked(true),
            KEY_SHOGUN_2 => app_ui.game_selected_shogun_2().set_checked(true),
            KEY_NAPOLEON => app_ui.game_selected_napoleon().set_checked(true),
            KEY_EMPIRE => app_ui.game_selected_empire().set_checked(true),
            _ => app_ui.game_selected_warhammer_3().set_checked(true),
        }

        // This may fail for path problems.
        //
        // Also, the game we already have loaded is arena. We don't need to force a manual reload with that one.
        if let Err(error) = app_ui.change_game_selected(false) {
            show_dialog(app_ui.main_window(), error, false);
        }

        app_ui.toggle_main_window(true);

        // If we have it enabled in the prefs, check if there are updates.
        if setting_bool("check_updates_on_start") {
            app_ui.check_updates(false);
        }

        // If we have it enabled in the prefs, check if there are schema updates.
        if setting_bool("check_schema_updates_on_start") {
            app_ui.check_schema_updates(false);
        };

        Ok(app_ui)
    }

    pub unsafe fn set_connections(&self, slots: &AppUISlots) {
        self.actions_ui().play_button().released().connect(slots.launch_game());
        self.actions_ui().enable_logging().toggled().connect(slots.toggle_logging());
        self.actions_ui().enable_skip_intro().toggled().connect(slots.toggle_skip_intros());
        self.actions_ui().merge_all_mods().toggled().connect(slots.toggle_merge_all_mods());
        self.actions_ui().enable_translations_combobox().current_text_changed().connect(slots.toggle_enable_translations());
        self.actions_ui().unit_multiplier_spinbox().value_changed().connect(slots.change_unit_multiplier());
        self.actions_ui().settings_button().released().connect(slots.open_settings());
        self.actions_ui().folders_button().released().connect(slots.open_folders_submenu());
        self.actions_ui().open_game_root_folder().triggered().connect(slots.open_game_root_folder());
        self.actions_ui().open_game_data_folder().triggered().connect(slots.open_game_data_folder());
        self.actions_ui().open_game_content_folder().triggered().connect(slots.open_game_content_folder());
        self.actions_ui().open_game_config_folder().triggered().connect(slots.open_game_config_folder());
        self.actions_ui().open_runcher_config_folder().triggered().connect(slots.open_runcher_config_folder());
        self.actions_ui().open_runcher_error_folder().triggered().connect(slots.open_runcher_error_folder());
        self.actions_ui().copy_load_order_button().released().connect(slots.copy_load_order());
        self.actions_ui().paste_load_order_button().released().connect(slots.paste_load_order());
        self.actions_ui().reload_button().released().connect(slots.reload());
        self.actions_ui().profile_load_button().released().connect(slots.load_profile());
        self.actions_ui().profile_save_button().released().connect(slots.save_profile());

        self.game_selected_pharaoh().triggered().connect(slots.change_game_selected());
        self.game_selected_warhammer_3().triggered().connect(slots.change_game_selected());
        self.game_selected_troy().triggered().connect(slots.change_game_selected());
        self.game_selected_three_kingdoms().triggered().connect(slots.change_game_selected());
        self.game_selected_warhammer_2().triggered().connect(slots.change_game_selected());
        self.game_selected_warhammer().triggered().connect(slots.change_game_selected());
        self.game_selected_thrones_of_britannia().triggered().connect(slots.change_game_selected());
        self.game_selected_attila().triggered().connect(slots.change_game_selected());
        self.game_selected_rome_2().triggered().connect(slots.change_game_selected());
        self.game_selected_shogun_2().triggered().connect(slots.change_game_selected());
        self.game_selected_napoleon().triggered().connect(slots.change_game_selected());
        self.game_selected_empire().triggered().connect(slots.change_game_selected());

        self.about_about_qt().triggered().connect(slots.about_qt());
        self.about_about_runcher().triggered().connect(slots.about_runcher());
        self.about_check_updates().triggered().connect(slots.check_updates());
        self.about_check_schema_updates().triggered().connect(slots.check_schema_updates());

        self.mod_list_ui().model().item_changed().connect(slots.update_pack_list());
        self.mod_list_ui().context_menu().about_to_show().connect(slots.mod_list_context_menu_open());
        self.mod_list_ui().enable_selected().triggered().connect(slots.enable_selected());
        self.mod_list_ui().disable_selected().triggered().connect(slots.disable_selected());
        self.mod_list_ui().category_delete().triggered().connect(slots.category_delete());
        self.mod_list_ui().category_rename().triggered().connect(slots.category_rename());
    }

    /// Function to toggle the main window on and off, while keeping the stupid focus from breaking.
    pub unsafe fn toggle_main_window(&self, enable: bool) {
        if enable {
            if *self.disabled_counter.read().unwrap() == 0 {
                error!("Bug: disabled counter broke. Needs investigation.");
            }

            if *self.disabled_counter.read().unwrap() > 0 {
                *self.disabled_counter.write().unwrap() -= 1;
            }

            if *self.disabled_counter.read().unwrap() == 0 && !self.main_window().is_enabled() {
                self.main_window().set_enabled(true);
                if let Some(focus_widget) = &*self.focused_widget.read().unwrap() {
                    if !focus_widget.is_null() && focus_widget.is_visible() && focus_widget.is_enabled() {
                        focus_widget.set_focus_0a();
                    }
                }

                *self.focused_widget.write().unwrap() = None;
            }
        }

        // Disabling, so store the focused widget. Do nothing if the window was already disabled.
        else {
            *self.disabled_counter.write().unwrap() += 1;
            if self.main_window().is_enabled() {
                let focus_widget = QApplication::focus_widget();
                if !focus_widget.is_null() {
                    *self.focused_widget.write().unwrap() = Some(focus_widget);
                }

                self.main_window().set_enabled(false);
            }
        }
    }

    pub unsafe fn change_game_selected(&self, reload_same_game: bool) -> Result<()> {

        // Get the new `Game Selected` and clean his name up, so it ends up like "x_y".
        let mut new_game_selected = self.game_selected_group.checked_action().text().to_std_string();
        if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
        let new_game_selected = new_game_selected.replace(' ', "_").to_lowercase();

        // If the game changed or we're initializing the program, change the game selected.
        //
        // This works because by default, the initially stored game selected is arena, and that one can never set manually.
        if reload_same_game || new_game_selected != self.game_selected().read().unwrap().key() {
            self.load_data(&new_game_selected)?;
        }

        Ok(())
    }

    pub unsafe fn load_data(&self, game: &str) -> Result<()> {

        // We may receive invalid games here, so rule out the invalid ones.
        match SUPPORTED_GAMES.game(game) {
            Some(game) => {

                // Schemas are optional, so don't interrupt loading due to they not being present.
                let schema_path = schemas_path().unwrap().join(game.schema_file_name());
                *SCHEMA.write().unwrap() = Schema::load(&schema_path, None).ok();
                *self.game_selected().write().unwrap() = game.clone();

                // Trigger an update of all game configs, just in case one needs update.
                let _ = GameConfig::update(game.key());

                // Load the game's config.
                *self.game_config().write().unwrap() = Some(GameConfig::load(game, true)?);

                // Load the profile's list.
                *self.game_profiles().write().unwrap() = Profile::profiles_for_game(game)?;
                self.actions_ui().profile_model().clear();

                for profile in self.game_profiles().read().unwrap().keys() {
                    self.actions_ui().profile_combobox().add_item_q_string(&QString::from_std_str(profile));
                }

                // If we don't have a path in the settings for the game, disable the play button.
                let game_path_str = setting_string(game.key());
                let game_path = PathBuf::from(&game_path_str);
                self.actions_ui().play_button().set_enabled(!game_path_str.is_empty());

                // Load the launch options for the game selected.
                setup_launch_options(self, game, &game_path);

                // Load the saves list for the selected game.
                if let Err(error) = self.load_saves_to_ui(game, &game_path) {
                    show_dialog(self.main_window(), error, false);
                }

                // Load the mods to the UI.
                if let Err(error) = self.load_mods_to_ui(game, &game_path) {
                    show_dialog(self.main_window(), error, false);
                }

                Ok(())
            },
            None => Err(anyhow!("Game {} is not a valid game.", game)),
        }
    }

    pub unsafe fn load_saves_to_ui(&self, game: &GameInfo, game_path: &Path) -> Result<()> {
        self.actions_ui().save_model().clear();
        let item = QStandardItem::from_q_string(&QString::from_std_str("No saves"));
        self.actions_ui().save_model().append_row_q_standard_item(item.into_ptr());

        // If we have a save folder for the game, read its saves and load them to the save combo.
        if let Some(ref config_path) = game.config_path(game_path) {
            let mut game_saves = self.game_saves.write().unwrap();
            game_saves.clear();

            let save_path = config_path.join("save_games");
            if let Ok(mut saves_paths) = files_from_subdir(&save_path, false) {

                // Sort them by date, then reverse, so the most recent one is first.
                saves_paths.sort_by_key(|x| x.metadata().unwrap().modified().unwrap());
                saves_paths.reverse();

                for save_path in &saves_paths {
                    let mut save = RFile::new_from_file_path(save_path)?;
                    save.guess_file_type()?;
                    if let Some(RFileDecoded::ESF(file)) = save.decode(&None, false, true)? {
                        let mut save = Save::default();
                        save.set_path(save_path.to_path_buf());
                        save.set_name(save_path.file_name().unwrap().to_string_lossy().to_string());
                        let mut mods = vec![];

                        let root_node = file.root_node();
                        if let NodeType::Record(node) = root_node {
                            if node.name() == "CAMPAIGN_SAVE_GAME" {
                                for children in node.children() {
                                    for child in children {
                                        if let NodeType::Record(node) = child {
                                            if node.name() == "SAVE_GAME_HEADER" {
                                                for children in node.children() {
                                                    for child in children {
                                                        if let NodeType::Record(node) = child {
                                                            if node.name() == "mod_history_block_name" {
                                                                for children in node.children() {
                                                                    if let NodeType::Ascii(pack_name) = &children[0] {
                                                                        //if let NodeType::Ascii(pack_folder) = &children[1] {
                                                                            mods.push(pack_name.to_owned());
                                                                        //}
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        save.set_mods(mods);

                        let item = QStandardItem::from_q_string(&QString::from_std_str(save.name()));
                        self.actions_ui().save_model().append_row_q_standard_item(item.into_ptr());

                        game_saves.push(save);
                    }
                }
            }
        }

        Ok(())
    }

    pub unsafe fn load_mods_to_ui(&self, game: &GameInfo, game_path: &Path) -> Result<()> {

        // Get the modified date of the game's exe, to check if a mod is outdated or not.
        let last_update_date = if let Some(exe_path) = game.executable_path(game_path) {
            if let Ok(exe) = File::open(exe_path) {
                exe.metadata()?.created()?.duration_since(UNIX_EPOCH)?.as_secs()
            } else {
                0
            }
        } else {
            0
        };

        let mut mods = self.game_config().write().unwrap();
        if let Some(ref mut mods) = *mods {

            // Clear the mod paths, just in case a failure while loading them leaves them unclean.
            mods.mods_mut().values_mut().for_each(|modd| modd.paths_mut().clear());

            // If we have a path, load all the mods to the UI.
            if game_path.components().count() > 1 && game_path.is_dir() {

                // Vanilla paths may fail if the game path is incorrect, or the game is not properly installed.
                // In that case, we assume there are no packs nor mods to load to avoid further errors.
                if let Ok(vanilla_packs) = game.ca_packs_paths(game_path) {
                    let data_paths = game.data_packs_paths(game_path);
                    let content_paths = game.content_packs_paths(game_path);

                    let mut steam_ids = vec![];

                    // Initialize the mods in the contents folders first.
                    //
                    // These have less priority.
                    if let Some(ref paths) = content_paths {
                        let packs = paths.par_iter()
                            .map(|path| (path, Pack::read_and_merge(&[path.to_path_buf()], true, false)))
                            .collect::<Vec<_>>();

                        for (path, pack) in packs {
                            let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                            if let Ok(pack) = pack {
                                if pack.pfh_file_type() == PFHFileType::Mod || pack.pfh_file_type() == PFHFileType::Movie {
                                    match mods.mods_mut().get_mut(&pack_name) {
                                        Some(modd) => {
                                            if !modd.paths().contains(path) {
                                                modd.paths_mut().push(path.to_path_buf());
                                            }

                                            // Get the steam id from the path, if possible.
                                            let steam_id = path.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();
                                            steam_ids.push(steam_id.to_owned());
                                            modd.set_steam_id(Some(steam_id));
                                            modd.set_pack_type(pack.pfh_file_type());

                                            let metadata = modd.paths().last().unwrap().metadata()?;
                                            modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_outdated(last_update_date > *modd.time_updated() as u64);
                                        }
                                        None => {
                                            let mut modd = Mod::default();
                                            modd.set_name(pack_name.to_owned());
                                            modd.set_id(pack_name.to_owned());
                                            modd.set_paths(vec![path.to_path_buf()]);
                                            modd.set_pack_type(pack.pfh_file_type());

                                            let metadata = modd.paths()[0].metadata()?;
                                            modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                            modd.set_outdated(last_update_date > *modd.time_updated() as u64);

                                            // Get the steam id from the path, if possible.
                                            let steam_id = path.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();
                                            steam_ids.push(steam_id.to_owned());
                                            modd.set_steam_id(Some(steam_id));

                                            mods.mods_mut().insert(pack_name, modd);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Ignore network population errors for now.
                    let _ = populate_mods(mods.mods_mut(), &steam_ids, last_update_date);

                    // If any of the mods has a .bin file, we need to copy it to /data and turn it into a Pack.
                    // All the if lets are because we only want to do all this if nothing files and ignore failures.
                    let steam_user_id = setting_string("steam_user_id");
                    for modd in mods.mods_mut().values_mut() {
                        if let Some(last_path) = modd.paths().last() {
                            if let Some(extension) = last_path.extension() {

                                // Only copy bins which are not yet in the data folder and which are not made by the steam user.
                                if extension.to_string_lossy() == "bin" && !modd.file_name().is_empty() {
                                    if let Ok(mut pack) = Pack::read_and_merge(&[last_path.to_path_buf()], true, false) {
                                        if let Ok(new_path) = game.data_path(game_path) {
                                            if let Some(name) = modd.file_name().split('/').last() {
                                                let new_path = new_path.join(name);

                                                // Copy the files unless it exists and its ours.
                                                if (!new_path.is_file() || (new_path.is_file() && &steam_user_id != modd.creator())) && pack.save(Some(&new_path), game, &None).is_ok() {
                                                    modd.paths_mut().insert(0, new_path);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(ref paths) = data_paths {
                        let paths = paths.iter()
                            .filter(|path| {
                                if let Ok(canon_path) = std::fs::canonicalize(path) {
                                    !vanilla_packs.contains(&canon_path) &&
                                        canon_path.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(|| String::new()) != RESERVED_PACK_NAME &&
                                        canon_path.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(|| String::new()) != RESERVED_PACK_NAME_ALTERNATIVE
                                } else {
                                    false
                                }
                            })
                            .collect::<Vec<_>>();

                        let packs = paths.par_iter()
                            .map(|path| (path, Pack::read_and_merge(&[path.to_path_buf()], true, false)))
                            .collect::<Vec<_>>();

                        for (path, pack) in packs {
                            let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                            if let Ok(pack) = pack {
                                if pack.pfh_file_type() == PFHFileType::Mod || pack.pfh_file_type() == PFHFileType::Movie {

                                    // Check if the pack corresponds to a bin.
                                    if let Some((_, modd)) = mods.mods_mut().iter_mut().find(|(_, modd)| !modd.file_name().is_empty() && modd.file_name().split('/').last().unwrap() == pack_name) {
                                        if !modd.paths().contains(path) {
                                            modd.paths_mut().insert(0, path.to_path_buf());
                                        }

                                        let metadata = modd.paths()[0].metadata()?;
                                        modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                        modd.set_outdated(last_update_date > *modd.time_updated() as u64);
                                    } else {
                                        match mods.mods_mut().get_mut(&pack_name) {
                                            Some(modd) => {
                                                if !modd.paths().contains(path) {
                                                    modd.paths_mut().insert(0, path.to_path_buf());
                                                }
                                                modd.set_pack_type(pack.pfh_file_type());

                                                let metadata = modd.paths()[0].metadata()?;
                                                modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                modd.set_outdated(last_update_date > *modd.time_updated() as u64);
                                            }
                                            None => {
                                                let mut modd = Mod::default();
                                                modd.set_name(pack_name.to_owned());
                                                modd.set_id(pack_name.to_owned());
                                                modd.set_paths(vec![path.to_path_buf()]);
                                                modd.set_pack_type(pack.pfh_file_type());

                                                let metadata = modd.paths()[0].metadata()?;
                                                modd.set_time_created(metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                modd.set_time_updated(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as usize);
                                                modd.set_outdated(last_update_date > *modd.time_updated() as u64);
                                                mods.mods_mut().insert(pack_name, modd);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Save the GameConfig or we may lost the population.
                    if let Err(error) = mods.save(game) {
                        show_dialog(self.main_window(), format!("Error while saving the mods info to disk: {}", error), false);
                    }
                }
            }

            self.mod_list_ui().load(mods)?;
            self.pack_list_ui().load(mods, game, game_path)?;
        }

        Ok(())
    }

    pub unsafe fn open_settings(&self) {
        let game_selected = self.game_selected().read().unwrap();
        let game_key = game_selected.key();
        let game_path_old = setting_path(game_key);

        match SettingsUI::new(self.main_window()) {
            Ok(saved) => {
                if saved {
                    let game_path_new = setting_path(game_key);

                    // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                    // re-select the current `GameSelected` to force it to reload the game's files.
                    if game_path_old != game_path_new {
                        QAction::trigger(&self.game_selected_group.checked_action());
                    }

                    // Disable the games we don't have a path for (uninstalled) and Shogun 2, as it's not supported yet.
                    for (_, game) in SUPPORTED_GAMES.games_sorted().iter().enumerate() {
                        let has_exe = game.executable_path(&setting_path(game.key())).filter(|path| path.is_file()).is_some();
                        match game.key() {
                            KEY_PHARAOH => self.game_selected_pharaoh().set_enabled(has_exe),
                            KEY_WARHAMMER_3 => self.game_selected_warhammer_3().set_enabled(has_exe),
                            KEY_TROY => self.game_selected_troy().set_enabled(has_exe),
                            KEY_THREE_KINGDOMS => self.game_selected_three_kingdoms().set_enabled(has_exe),
                            KEY_WARHAMMER_2 => self.game_selected_warhammer_2().set_enabled(has_exe),
                            KEY_WARHAMMER => self.game_selected_warhammer().set_enabled(has_exe),
                            KEY_THRONES_OF_BRITANNIA => self.game_selected_thrones_of_britannia().set_enabled(has_exe),
                            KEY_ATTILA => self.game_selected_attila().set_enabled(has_exe),
                            KEY_ROME_2 => self.game_selected_rome_2().set_enabled(has_exe),
                            KEY_SHOGUN_2 => self.game_selected_shogun_2().set_enabled(has_exe),
                            KEY_NAPOLEON => self.game_selected_napoleon().set_enabled(has_exe),
                            KEY_EMPIRE => self.game_selected_empire().set_enabled(has_exe),
                            _ => {},
                        }
                    }

                    // If we detect a factory reset, reset the window's geometry and state, and the font.
                    let factory_reset = setting_bool("factoryReset");
                    if factory_reset {
                        self.main_window().restore_geometry(&setting_byte_array("originalGeometry"));
                        self.main_window().restore_state_1a(&setting_byte_array("originalWindowState"));
                    }
                }
            }
            Err(error) => show_dialog(&self.main_window, error, false),
        }

        // Make sure we don't drag the factory reset setting, no matter if the user saved or not.
        set_setting_bool("factoryReset", false);
    }

    pub unsafe fn launch_game(&self) -> Result<()> {
        let mut folder_list = String::new();
        let mut pack_list = String::new();
        let game = self.game_selected().read().unwrap();
        let game_path = setting_path(game.key());
        let data_path = game.data_path(&game_path)?;

        // We only use the reserved pack if we need to.
        if (self.actions_ui().enable_logging().is_enabled() && self.actions_ui().enable_logging().is_checked()) ||
            (self.actions_ui().enable_skip_intro().is_enabled() && self.actions_ui().enable_skip_intro().is_checked()) ||
            (self.actions_ui().enable_translations_combobox().is_enabled() && self.actions_ui().enable_translations_combobox().current_index() != 0) ||
            (self.actions_ui().unit_multiplier_spinbox().is_enabled() && self.actions_ui().unit_multiplier_spinbox().value() != 1.00) {

            // We need to use an alternative name for Shogun 2, Rome 2, Attila and Thrones because their load order logic for movie packs seems... either different or broken.
            let reserved_pack_name = if game.key() == KEY_SHOGUN_2 || game.key() == KEY_ROME_2 || game.key() == KEY_ATTILA || game.key() == KEY_THRONES_OF_BRITANNIA { RESERVED_PACK_NAME_ALTERNATIVE } else { RESERVED_PACK_NAME };

            // Support for add_working_directory seems to be only present in rome 2 and newer games. For older games, we drop the pack into /data.
            let temp_path = if *game.raw_db_version() >= 2 {
                let temp_packs_folder = temp_packs_folder(&game)?;
                let temp_path = temp_packs_folder.join(&reserved_pack_name);
                folder_list.push_str(&format!("add_working_directory \"{}\";\n", temp_packs_folder.to_string_lossy()));
                temp_path
            } else {
                data_path.join(&reserved_pack_name)
            };

            // Generate the reserved pack.
            //
            // Note: It has to be a movie pack because otherwise we cannot overwrite the intro files in older games.
            let pack_version = game.pfh_version_by_file_type(PFHFileType::Movie);
            let mut reserved_pack = Pack::new_with_version(pack_version);
            reserved_pack.set_pfh_file_type(PFHFileType::Movie);

            // Skip videos.
            prepare_skip_intro_videos(self, &game, &game_path, &mut reserved_pack)?;

            // Logging.
            prepare_script_logging(self, &game, &mut reserved_pack)?;

            // Translations.
            prepare_translations(self, &game, &mut reserved_pack)?;

            // Unit multiplier.
            prepare_unit_multiplier(self, &game, &game_path, &mut reserved_pack)?;

            let mut encode_data = EncodeableExtraData::default();
            encode_data.set_nullify_dates(true);

            reserved_pack.save(Some(&temp_path), &game, &Some(encode_data))?;
        }

        // If we have "merge all mods" checked, we need to load the entire load order into a single pack, and load that pack instead of the entire load order.
        if self.actions_ui().merge_all_mods().is_enabled() && self.actions_ui().merge_all_mods().is_checked() {
            let temp_path_file_name = format!("{}_{}.pack", MERGE_ALL_PACKS_PACK_NAME, self.game_selected().read().unwrap().key());
            let temp_path = data_path.join(&temp_path_file_name);
            pack_list.push_str(&format!("mod \"{}\";", temp_path_file_name));

            // Generate the merged pack.
            let pack_paths = (0..self.pack_list_ui().model().row_count_0a())
                .filter_map(|index| {
                    let item_type = self.pack_list_ui().model().item_2a(index, 1);
                    let item_path = self.pack_list_ui().model().item_2a(index, 2);

                    if item_type.text().to_std_string() == "Mod" {
                        let path = PathBuf::from(item_path.text().to_std_string());

                        // Canonicalization is required due to some issues with the game not loading not properly formatted paths.
                        std::fs::canonicalize(path).ok()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !pack_paths.is_empty() {
                let mut reserved_pack = Pack::read_and_merge(&pack_paths, true, false)?;
                let pack_version = game.pfh_version_by_file_type(PFHFileType::Mod);
                reserved_pack.set_pfh_version(pack_version);

                let mut encode_data = EncodeableExtraData::default();
                encode_data.set_nullify_dates(true);

                reserved_pack.save(Some(&temp_path), &game, &Some(encode_data))?;
            }
        }

        // Otherwise, just add the packs from the load order to the text file.
        else {

            // TODO: Replace this with the load order list once it's made global.
            pack_list.push_str(&(0..self.pack_list_ui().model().row_count_0a())
                .filter_map(|index| {
                    let mut string = String::new();
                    let item = self.pack_list_ui().model().item_1a(index);
                    let item_type = self.pack_list_ui().model().item_2a(index, 1);
                    let item_path = self.pack_list_ui().model().item_2a(index, 2);
                    let item_location = self.pack_list_ui().model().item_2a(index, 4);
                    let item_steam_id = self.pack_list_ui().model().item_2a(index, 5);

                    if item_type.text().to_std_string() == "Mod" {
                        let steam_id = item_steam_id.text().to_std_string();

                        // Loading packs from outside /data is only supported on rome 2 and newer games.
                        if item_location.text().to_std_string().starts_with("Content") && *game.raw_db_version() >= 2 && !steam_id.is_empty() {
                            let mut path = PathBuf::from(item_path.text().to_std_string());
                            path.pop();

                            // Canonicalization is required due to some issues with the game not loading not properly formatted paths.
                            if let Ok(path) = std::fs::canonicalize(path) {
                                let mut path_str = path.to_string_lossy().to_string();
                                if path_str.starts_with("\\\\?\\") {
                                    path_str = path_str[4..].to_string();
                                }

                                folder_list.push_str(&format!("add_working_directory \"{}\";\n", path_str));
                            } else {
                                return None;
                            }
                        }

                        string.push_str(&format!("mod \"{}\";", item.text().to_std_string()));
                        Some(string)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"));
        }


        // Check if we are loading a save. First option is no save load. Any index above that is a save.
        let mut extra_args = vec![];
        let save_index = self.actions_ui.save_combobox().current_index();
        if self.actions_ui.save_combobox().current_index() > 0 {
            if let Some(save) = self.game_saves.read().unwrap().get(save_index as usize - 1) {
                extra_args.push("game_startup_mode".to_owned());
                extra_args.push("campaign_load".to_owned());
                extra_args.push(save.name().to_owned());
            }
        }

        // NOTE: On Empire and Napoleon we need to use the user_script, not the custom file, as it doesn't seem to work.
        // Older versions of shogun 2 also used the user_script, but the latest update enabled use of custom mod lists.
        let file_path = if *game.raw_db_version() >= 1 {
            game_path.join(CUSTOM_MOD_LIST_FILE_NAME)
        } else {

            // Games may fail to launch if we don't have this path created, which is done the first time we start the game.
            let config_path = game.config_path(&game_path).ok_or(anyhow!("Error getting the game's config path."))?;
            let scripts_path = config_path.join("scripts");
            DirBuilder::new().recursive(true).create(&scripts_path)?;

            scripts_path.join(USER_SCRIPT_FILE_NAME)
        };

        let mut file = BufWriter::new(File::create(file_path)?);

        // Napoleon, Empire and Shogun 2 require the user.script.txt or mod list file (for Shogun's latest update) to be in UTF-16 LE. What the actual fuck.
        if *game.raw_db_version() < 2 {
            file.write_string_u16(&folder_list)?;
            file.write_string_u16(&pack_list)?;
        } else {
            file.write_all(folder_list.as_bytes())?;
            file.write_all(pack_list.as_bytes())?;
        }

        file.flush()?;

        match game.executable_path(&game_path) {
            Some(exec_game) => {
                if cfg!(target_os = "windows") {

                    // For post-shogun 2 games, we use the same command to bypass the launcher.
                    if *game.raw_db_version() >= 2 {

                        let mut command = SystemCommand::new("cmd");
                        command.arg("/C");
                        command.arg("start");
                        command.arg("/d");
                        command.arg(game_path.to_string_lossy().replace('\\', "/"));
                        command.arg(exec_game.file_name().unwrap().to_string_lossy().to_string());
                        command.arg(CUSTOM_MOD_LIST_FILE_NAME.to_string() + ";");

                        for arg in &extra_args {
                            command.arg(arg);
                        }

                        // This disables the terminal when executing the command.
                        #[cfg(target_os = "windows")]command.creation_flags(CREATE_NO_WINDOW);
                        command.spawn()?;
                    }

                    // Empire and Napoleon do not have a launcher. We can make our lives easier calling steam instead of launching the game manually.
                    else if *game.raw_db_version() == 0 {
                        match game.game_launch_command(&game_path) {
                            Ok(command) => { let _ = open::that(command); },
                            _ => show_dialog(self.main_window(), "The currently selected game cannot be launched from Steam.", false),
                        }
                    }

                    // Shogun 2 has problems since we lost the hot ashigaru sex chat. The current theory I have is that launching from the exe seems to skip the steam checks,
                    // meaning the game launches as if you do not own anything on it. Nor the base game nor the dlcs. Launching from the launcher works, as well as launching from steam.
                    //
                    // The only method I found that works is replacing the vanilla launcher with an exe that bounces of to the exe, so we launch the game from steam,
                    // which does the ownership checks, that launches our custom launcher, which launches the exe of the game with the custom mod list.
                    //
                    // Also, is not my idea. Someone already did it on steam with an aut2exe script.
                    else {
                        let mut launcher_path = game_path.join("launcher");
                        let mut launcher_path_bak = launcher_path.to_path_buf();
                        launcher_path.push("launcher.exe");
                        launcher_path_bak.push("launcher.exe.bak");

                        // On debug mode, it's in third party libs. On release, it's in runcher's folder.
                        let mut bouncer_path = std::env::current_exe()?;
                        if cfg!(debug_assertions) {
                            bouncer_path.pop();
                            bouncer_path.pop();
                            bouncer_path.pop();
                            bouncer_path.push("3rdparty");
                            bouncer_path.push("builds");
                        } else {
                            bouncer_path.pop();
                        }
                        bouncer_path.push("bouncer.exe");

                        let replace_launcher = if let Ok(file) = File::open(&launcher_path) {
                            if let Ok(metadata) = file.metadata() {

                                // Vanilla launcher is about 50mb, bouncer is less than one.
                                if cfg!(target_os = "windows") {
                                    metadata.file_size() > 1_000_000
                                } else {
                                    metadata.len() > 1_000_000
                                }
                            } else {
                                true
                            }
                        } else {
                            true
                        };

                        // If this fails, report it.
                        if replace_launcher {
                            copy(&launcher_path, launcher_path_bak)?;
                            copy(bouncer_path, launcher_path)?;
                        }

                        // Once we've replaced the launcher (if needed), launch the game from steam.
                        match game.game_launch_command(&game_path) {
                            Ok(command) => { let _ = open::that(command); },
                            _ => show_dialog(self.main_window(), "The currently selected game cannot be launched from Steam.", false),
                        }
                    }

                    Ok(())
                } else if cfg!(target_os = "linux") {
                    Err(anyhow!("Unsupported OS."))
                } else {
                    Err(anyhow!("Unsupported OS."))
                }
            }
            None => Err(anyhow!("Executable path not found. Is the game folder configured correctly in the settings?"))
        }
    }

    pub unsafe fn load_profile(&self) -> Result<()> {
        let profile_name = self.actions_ui().profile_combobox().current_text().to_std_string();
        if profile_name.is_empty() {
            return Err(anyhow!("Profile name is empty."));
        }

        match self.game_profiles().read().unwrap().get(&profile_name) {
            Some(profile) => {

                // First, disable all mods.
                for cat in 0..self.mod_list_ui().model().row_count_0a() {
                    let category = self.mod_list_ui().model().item_1a(cat);
                    for row in 0..category.row_count() {
                        let item = category.child_1a(row);
                        item.set_check_state(CheckState::Unchecked);
                    }
                }

                for mod_id in profile.mods() {
                    let mod_id = QString::from_std_str(mod_id);
                    for cat in 0..self.mod_list_ui().model().row_count_0a() {
                        let category = self.mod_list_ui().model().item_1a(cat);
                        for row in 0..category.row_count() {
                            let item = category.child_1a(row);
                            if !item.is_null() && item.data_1a(VALUE_MOD_ID).to_string().compare_q_string(&mod_id) == 0 {
                                item.set_check_state(CheckState::Checked);
                            }
                        }
                    }
                }

                Ok(())
            }
            None => Err(anyhow!("No profile with said name found."))
        }
    }

    pub unsafe fn save_profile(&self) -> Result<()> {
        let profile_name = self.actions_ui().profile_combobox().current_text().to_std_string();
        if profile_name.is_empty() {
            return Err(anyhow!("Profile name is empty."));
        }

        if let Some(ref game_config) = *self.game_config().read().unwrap() {

            let mods = game_config.mods()
                .values()
                .filter_map(|modd| if *modd.enabled() { Some(modd.id().to_string()) } else { None })
                .collect::<Vec<_>>();

            let mut profile = Profile::default();
            profile.set_id(profile_name.to_owned());
            profile.set_mods(mods);

            self.game_profiles().write().unwrap().insert(profile_name.to_owned(), profile.clone());

            self.actions_ui().profile_model().clear();
            for profile in self.game_profiles().read().unwrap().keys() {
                self.actions_ui().profile_combobox().add_item_q_string(&QString::from_std_str(profile));
            }

            return profile.save(&self.game_selected().read().unwrap(), &profile_name);
        }

        Ok(())
    }

    pub unsafe fn mod_list_selection(&self) -> Vec<CppBox<QModelIndex>> {
        self.mod_list_ui().mod_list_selection()
    }

    /// This function checks if there is any newer version of the app released.
    ///
    /// If the `use_dialog` is false, we make the checks in the background, and pop up a dialog only in case there is an update available.
    pub unsafe fn check_updates(&self, use_dialog: bool) {
        let receiver = CENTRAL_COMMAND.send_network(Command::CheckUpdates);

        let dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
            q_message_box::Icon::Information,
            &qtr("update_checker"),
            &qtr("update_searching"),
            QFlags::from(q_message_box::StandardButton::Close),
            self.main_window(),
        );

        let close_button = dialog.button(q_message_box::StandardButton::Close);
        let update_button = dialog.add_button_q_string_button_role(&qtr("update_button"), q_message_box::ButtonRole::AcceptRole);
        update_button.set_enabled(false);

        dialog.set_modal(true);
        if use_dialog {
            dialog.show();
        }

        let response = CENTRAL_COMMAND.recv_try(&receiver);
        let message = match response {
            Response::APIResponse(response) => {
                match response {
                    APIResponse::NewStableUpdate(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_stable_update", &[&last_release])
                    }
                    APIResponse::NewBetaUpdate(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_beta_update", &[&last_release])
                    }
                    APIResponse::NewUpdateHotfix(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_update_hotfix", &[&last_release])
                    }
                    APIResponse::NoUpdate => {
                        if !use_dialog { return; }
                        qtr("api_response_success_no_update")
                    }
                    APIResponse::UnknownVersion => {
                        if !use_dialog { return; }
                        qtr("api_response_success_unknown_version")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };

        dialog.set_text(&message);
        if dialog.exec() == 0 {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateMainProgram);

            dialog.show();
            dialog.set_text(&qtr("update_in_prog"));
            update_button.set_enabled(false);
            close_button.set_enabled(false);

            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::Success => {
                    let restart_button = dialog.add_button_q_string_button_role(&qtr("restart_button"), q_message_box::ButtonRole::ApplyRole);

                    let changelog_path = PROGRAM_PATH.join(CHANGELOG_FILE);
                    dialog.set_text(&qtre("update_success_main_program", &[&changelog_path.to_string_lossy()]));
                    restart_button.set_enabled(true);
                    close_button.set_enabled(true);

                    // This closes the program and triggers a restart.
                    if dialog.exec() == 1 {

                        // Make sure we close both threads and the window. In windows the main window doesn't get closed for some reason.
                        CENTRAL_COMMAND.send_background(Command::Exit);
                        CENTRAL_COMMAND.send_network(Command::Exit);
                        QApplication::close_all_windows();

                        let rpfm_exe_path = current_exe().unwrap();
                        SystemCommand::new(rpfm_exe_path).spawn().unwrap();
                        exit(10);
                    }
                },
                Response::Error(error) => {
                    dialog.set_text(&QString::from_std_str(error.to_string()));
                    close_button.set_enabled(true);
                }
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }
    }

    /// This function checks if there is any newer version of RPFM's schemas released.
    ///
    /// If the `use_dialog` is false, we only show a dialog in case of update available. Useful for checks at start.
    pub unsafe fn check_schema_updates(&self, use_dialog: bool) {
        let receiver = CENTRAL_COMMAND.send_network(Command::CheckSchemaUpdates);

        // Create the dialog to show the response and configure it.
        let dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
            q_message_box::Icon::Information,
            &qtr("update_schema_checker"),
            &qtr("update_searching"),
            QFlags::from(q_message_box::StandardButton::Close),
            self.main_window(),
        );

        let close_button = dialog.button(q_message_box::StandardButton::Close);
        let update_button = dialog.add_button_q_string_button_role(&qtr("update_button"), q_message_box::ButtonRole::AcceptRole);
        update_button.set_enabled(false);

        dialog.set_modal(true);
        if use_dialog {
            dialog.show();
        }

        // When we get a response, act depending on the kind of response we got.
        let response_thread = CENTRAL_COMMAND.recv_try(&receiver);
        let message = match response_thread {
            Response::APIResponseGit(ref response) => {
                match response {
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_button.set_enabled(true);
                        qtr("schema_new_update")
                    }
                    GitResponse::NoUpdate => {
                        if !use_dialog { return; }
                        qtr("schema_no_update")
                    }
                    GitResponse::NoLocalFiles => {
                        update_button.set_enabled(true);
                        qtr("update_no_local_schema")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response_thread:?}"),
        };

        // If we hit "Update", try to update the schemas.
        dialog.set_text(&message);
        if dialog.exec() == 0 {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateSchemas(self.game_selected().read().unwrap().schema_file_name().to_owned()));

            dialog.show();
            dialog.set_text(&qtr("update_in_prog"));
            update_button.set_enabled(false);
            close_button.set_enabled(false);

            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::Success => {

                    // We need to reload the game in question, so stuff that depends on schemas existing actually works.
                    if let Err(error) = self.change_game_selected(true) {
                        show_dialog(&dialog, error, false);
                    }

                    dialog.set_text(&qtr("schema_update_success"));
                    close_button.set_enabled(true);
                },
                Response::Error(error) => {
                    dialog.set_text(&QString::from_std_str(error.to_string()));
                    close_button.set_enabled(true);
                }
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }
    }

    /// This function creates the stylesheet used for the dark theme in windows.
    pub fn dark_stylesheet() -> Result<String> {
        let mut file = File::open(ASSETS_PATH.join("dark-theme.qss"))?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        Ok(string.replace("{assets_path}", &ASSETS_PATH.to_string_lossy().replace('\\', "/")))
    }

    /// This function is used to load/reload a theme live.
    pub unsafe fn reload_theme() {
        let app = QCoreApplication::instance();
        let qapp = app.static_downcast::<QApplication>();
        let use_dark_theme = setting_bool("dark_mode");

        // Initialize the globals before applying anything.
        let light_style_sheet = ref_from_atomic(&*LIGHT_STYLE_SHEET);
        let light_palette = ref_from_atomic(&*LIGHT_PALETTE);
        let dark_palette = ref_from_atomic(&*DARK_PALETTE);

        // On Windows, we use the dark theme switch to control the Style, StyleSheet and Palette.
        if cfg!(target_os = "windows") {
            if use_dark_theme {
                QApplication::set_style_q_string(&QString::from_std_str("fusion"));
                QApplication::set_palette_1a(dark_palette);
                if let Ok(dark_stylesheet) = Self::dark_stylesheet() {
                    qapp.set_style_sheet(&QString::from_std_str(dark_stylesheet));
                }
            } else {
                QApplication::set_style_q_string(&QString::from_std_str("windowsvista"));
                QApplication::set_palette_1a(light_palette);
                qapp.set_style_sheet(light_style_sheet);
            }
        }

        // On MacOS, we use the dark theme switch to control the StyleSheet and Palette.
        else if cfg!(target_os = "macos") {
            if use_dark_theme {
                QApplication::set_palette_1a(dark_palette);
                if let Ok(dark_stylesheet) = Self::dark_stylesheet() {
                    qapp.set_style_sheet(&QString::from_std_str(dark_stylesheet));
                }
            } else {
                QApplication::set_palette_1a(light_palette);
                qapp.set_style_sheet(light_style_sheet);
            }
        }

        // Linux and company.
        else if use_dark_theme {
            qt_widgets::QApplication::set_palette_1a(dark_palette);
            if let Ok(dark_stylesheet) = Self::dark_stylesheet() {
                qapp.set_style_sheet(&QString::from_std_str(dark_stylesheet));
            }
        } else {
            qt_widgets::QApplication::set_palette_1a(light_palette);
            qapp.set_style_sheet(light_style_sheet);
        }
    }

    pub unsafe fn load_order_string_dialog(&self, string: Option<String>) -> Result<Option<String>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { LOAD_ORDER_STRING_VIEW_DEBUG } else { LOAD_ORDER_STRING_VIEW_RELEASE };
        let main_widget = load_template(self.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();

        let info_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "string_label")?;
        let string_text_edit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "string_text_edit")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        if let Some(ref string) = string {
            dialog.set_window_title(&qtr("load_order_string_title_copy"));
            info_label.set_text(&qtr("load_order_string_info_copy"));
            string_text_edit.set_text(&QString::from_std_str(string));
        } else {
            dialog.set_window_title(&qtr("load_order_string_title_paste"));
            info_label.set_text(&qtr("load_order_string_info_paste"));
        }

        // If we're in "receive" mode, add a cancel button.
        if string.is_none() {
            button_box.add_button_standard_button(StandardButton::Cancel);
        }

        if dialog.exec() == 1 && string.is_none() {
            Ok(Some(string_text_edit.to_plain_text().to_std_string()))
        } else {
            Ok(None)
        }
    }

    pub unsafe fn load_order_from_shareable_mod_list(&self, shareable_mod_list: &[ShareableMod]) -> Result<()> {
        if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
            let mut missing = vec![];
            let mut wrong_hash = vec![];
            for modd in shareable_mod_list {
                match game_config.mods_mut().get_mut(modd.id()) {
                    Some(modd_local) => {
                        let current_hash = try_digest(modd_local.paths()[0].as_path())?;
                        if &current_hash != modd.hash() {
                            wrong_hash.push(modd.clone());
                        }

                        modd_local.set_enabled(true);
                    },
                    None => missing.push(modd.clone()),
                }
            }

            // Report any missing mods.
            if !missing.is_empty() || !wrong_hash.is_empty() {
                let mut message = String::new();

                if !missing.is_empty() {
                    message.push_str(&format!("<p>The following mods have not been found in the mod list:<p> <ul>{}</ul>",
                        missing.iter().map(|modd| match modd.steam_id() {
                            Some(steam_id) => format!("<li>{}: <a src=\"https://steamcommunity.com/sharedfiles/filedetails/?id={}\">{}</a></li>", modd.id(), steam_id, modd.name()),
                            None => format!("<li>{}</li>", modd.id())
                        }).collect::<Vec<_>>().join("\n")
                    ));
                }

                if !wrong_hash.is_empty() {
                    message.push_str(&format!("<p>The following mods have been found, but their packs are different from the ones expected:<p> <ul>{}</ul>",
                        wrong_hash.iter().map(|modd| match modd.steam_id() {
                            Some(steam_id) => format!("<li>{}: <a src=\"https://steamcommunity.com/sharedfiles/filedetails/?id={}\">{}</a></li>", modd.id(), steam_id, modd.name()),
                            None => format!("<li>{}</li>", modd.id())
                        }).collect::<Vec<_>>().join("\n")
                    ));
                }
                show_dialog(self.main_window(), message, false);
            }

            let game = self.game_selected().read().unwrap();
            let game_path = setting_path(game.key());
            self.mod_list_ui().load(game_config)?;
            self.pack_list_ui().load(game_config, &game, &game_path)?;

            game_config.save(&game)?;
        }

        Ok(())
    }

    pub unsafe fn delete_category(&self) -> Result<()> {
        let mut selection = self.mod_list_selection();
        selection.sort_by_key(|b| Reverse(b.row()));

        if selection.iter().any(|index| index.data_1a(2).to_string().to_std_string() == "Unassigned") {
            return Err(anyhow!("Dude, did you just tried to delete the Unassigned category?!! You monster!!!"));
        }

        for cat_to_delete in &selection {
            let mods_to_reassign = (0..self.mod_list_ui().model().row_count_1a(cat_to_delete))
                .map(|index| cat_to_delete.child(index, 0).data_1a(VALUE_MOD_ID).to_string().to_std_string())
                .collect::<Vec<_>>();

            if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
                game_config.mods_mut()
                    .iter_mut()
                    .for_each(|(id, modd)| if mods_to_reassign.contains(id) {
                        modd.set_category(None);
                    });
            }

            // Find the unassigned category.
            let mut unassigned_item = None;
            let unassigned = QString::from_std_str("Unassigned");
            for index in 0..self.mod_list_ui().model().row_count_0a() {
                let item = self.mod_list_ui().model().item_1a(index);
                if !item.is_null() && item.text().compare_q_string(&unassigned) == 0 {
                    unassigned_item = Some(item);
                    break;
                }
            }

            if let Some(unassigned_item) = unassigned_item {
                let cat_item = self.mod_list_ui().model().item_from_index(cat_to_delete);
                for index in (0..self.mod_list_ui().model().row_count_1a(cat_to_delete)).rev() {
                    let taken = cat_item.take_row(index).into_ptr();
                    unassigned_item.append_row_q_list_of_q_standard_item(taken.as_ref().unwrap());
                }
            }

            self.mod_list_ui().model().remove_row_1a(cat_to_delete.row());
        }

        let game_info = self.game_selected().read().unwrap();
        if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
            game_config.save(&game_info)?;
        }

        Ok(())
    }

    pub unsafe fn rename_category(&self) -> Result<()> {
        if let Some(new_cat_name) = self.mod_list_ui().category_new_dialog(true)? {
            let selection = self.mod_list_selection();
            let cat_index = &selection[0];
            let old_cat_name = cat_index.data_1a(2).to_string().to_std_string();
            if old_cat_name == "Unassigned" {
                return Err(anyhow!("Dude, did you just tried to rename the Unassigned category?!! You cannot rename perfection!!!"));
            }

            let cat_item = self.mod_list_ui().model().item_from_index(cat_index);
            cat_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&new_cat_name)), 2);

            if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
                game_config.mods_mut()
                    .values_mut()
                    .for_each(|modd| if let Some(ref mut old_cat) = modd.category_mut() {
                        if *old_cat == old_cat_name {
                            *old_cat = new_cat_name.to_owned();
                        }
                    });
            }

            let game_info = self.game_selected().read().unwrap();
            if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
                game_config.save(&game_info)?;
            }
        }
        Ok(())
    }

    pub unsafe fn generate_move_to_category_submenu(app_ui: &Arc<AppUI>) {
        app_ui.mod_list_ui().categories_send_to_menu().clear();

        let categories = app_ui.mod_list_ui().categories();
        for category in &categories {

            let item = app_ui.mod_list_ui().category_item(category);
            if let Some(item) = item {
                let action = app_ui.mod_list_ui().categories_send_to_menu().add_action_q_string(&QString::from_std_str(category));
                let slot = SlotNoArgs::new(app_ui.mod_list_ui().categories_send_to_menu(), clone!(
                    category,
                    app_ui => move || {
                        let mut selection = app_ui.mod_list_selection();
                        selection.sort_by_key(|b| Reverse(b.row()));

                        for mod_item in &selection {
                            let current_cat = mod_item.parent();
                            let mod_id = mod_item.data_1a(VALUE_MOD_ID).to_string().to_std_string();
                            let taken = app_ui.mod_list_ui().model().item_from_index(&current_cat).take_row(mod_item.row()).into_ptr();
                            item.append_row_q_list_of_q_standard_item(taken.as_ref().unwrap());

                            if let Some(ref mut game_config) = *app_ui.game_config().write().unwrap() {
                                if let Some(ref mut modd) = game_config.mods_mut().get_mut(&mod_id) {
                                    modd.set_category(Some(category.to_string()));
                                }

                                let game_info = app_ui.game_selected().read().unwrap();
                                if let Err(error) = game_config.save(&game_info) {
                                    show_dialog(app_ui.main_window(), error, false);
                                }
                            }
                        }
                    }
                ));

                action.triggered().connect(&slot);
            }
        }
    }
}
