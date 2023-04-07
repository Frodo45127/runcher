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
use qt_widgets::{QDialog, QDialogButtonBox, q_dialog_button_box::StandardButton};
use qt_widgets::QLabel;
use qt_widgets::QMainWindow;
use qt_widgets::QSplitter;
use qt_widgets::QTextEdit;
use qt_widgets::{QMessageBox, q_message_box};
use qt_widgets::QWidget;

use qt_gui::QIcon;

use qt_core::CheckState;
use qt_core::QBox;
use qt_core::QCoreApplication;
use qt_core::QFlags;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::SlotNoArgs;

use cpp_core::CppBox;

use anyhow::{anyhow, Result};
use getset::Getters;
use sha256::try_digest;

use std::env::{args, current_exe};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
#[cfg(target_os = "windows")] use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command as SystemCommand, exit};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::files::{RFile, FileType, pack::Pack};
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType, supported_games::*};
use rpfm_lib::integrations::log::*;

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
use crate::integrations::{GameConfig, Mod, Profile, ShareableMod, steam::*};
use crate::LIGHT_PALETTE;
use crate::LIGHT_STYLE_SHEET;
use crate::mod_list_ui::*;
use crate::pack_list_ui::PackListUI;
use crate::settings_ui::SettingsUI;
use crate::settings_ui::init_settings;
use crate::SUPPORTED_GAMES;
use crate::updater::*;

use self::slots::AppUISlots;

pub mod slots;

#[cfg(target_os = "windows")] const CREATE_NO_WINDOW: u32 = 0x08000000;
//const DETACHED_PROCESS: u32 = 0x00000008;

const LOAD_ORDER_STRING_VIEW_DEBUG: &str = "ui_templates/load_order_string_dialog.ui";
const LOAD_ORDER_STRING_VIEW_RELEASE: &str = "ui/load_order_string_dialog.ui";

const EMPTY_MOVIE: [u8; 595] = [
    0x43, 0x41, 0x4d, 0x56, 0x01, 0x00, 0x29, 0x00, 0x56, 0x50, 0x38, 0x30, 0x80, 0x02, 0xe0, 0x01, 0x55, 0x55,
    0x85, 0x42, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x4a, 0x02, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x21, 0x02, 0x00, 0x00, 0x00, 0x50, 0x42, 0x00, 0x9d, 0x01, 0x2a, 0x80, 0x02, 0xe0, 0x01, 0x00, 0x47, 0x08,
    0x85, 0x85, 0x88, 0x85, 0x84, 0x88, 0x02, 0x02, 0x00, 0x06, 0x16, 0x04, 0xf7, 0x06, 0x81, 0x64, 0x9f, 0x6b,
    0xdb, 0x9b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x37, 0x80, 0xfe, 0xff, 0xab, 0x50, 0x80, 0x29, 0x00, 0x00, 0x00, 0x21, 0x02, 0x00, 0x00,
    0x01,
];

const INTRO_MOVIE_PATHS_BY_GAME: [(&str, [&str; 6]); 1] = [
    ("warhammer_3",
        [
            "movies/epilepsy_warning/epilepsy_warning_en.ca_vp8",
            "movies/gam_int.ca_vp8",
            "movies/startup_movie_01.ca_vp8",
            "movies/startup_movie_02.ca_vp8",
            "movies/startup_movie_03.ca_vp8",
            "movies/startup_movie_04.ca_vp8",
        ]
    )
];

const SCRIPT_DEBUG_ACTIVATOR_PATH: &str = "script/enable_console_logging";
const RESERVED_PACK_NAME: &str = "!!!!!!!!!!!!!!!!!!!!!run_you_fool";

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
        let widget = QWidget::new_1a(&main_window);
        let central_layout = create_grid_layout(widget.static_upcast());
        main_window.set_central_widget(&widget);
        main_window.resize_2a(1300, 800);
        main_window.set_window_title(&QString::from_std_str("The Runcher"));
        QApplication::set_window_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/runcher.png", ASSETS_PATH.to_string_lossy()))));

        let splitter = QSplitter::from_q_widget(&widget);
        let left_widget = QWidget::new_1a(&splitter);
        let right_widget = QWidget::new_1a(&splitter);
        let _ = create_grid_layout(left_widget.static_upcast());
        let _ = create_grid_layout(right_widget.static_upcast());
        splitter.set_stretch_factor(0, 1);
        right_widget.set_minimum_width(540);

        central_layout.add_widget(splitter.into_raw_ptr());

        // Get the menu and status bars.
        let menu_bar = main_window.menu_bar();
        let status_bar = main_window.status_bar();
        status_bar.set_size_grip_enabled(false);
        let menu_bar_game_selected = menu_bar.add_menu_q_string(&qtr("menu_bar_game_selected"));
        let menu_bar_about = menu_bar.add_menu_q_string(&qtr("menu_bar_about"));

        //-----------------------------------------------//
        // `Game Selected` Menu.
        //-----------------------------------------------//
        let game_selected_warhammer_3 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER_3));
        let game_selected_troy = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_TROY));
        let game_selected_three_kingdoms = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_THREE_KINGDOMS));
        let game_selected_warhammer_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER_2));
        let game_selected_warhammer = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER));
        let game_selected_thrones_of_britannia = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_THRONES_OF_BRITANNIA));
        let game_selected_attila = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_ATTILA));
        let game_selected_rome_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_ROME_2));
        let game_selected_shogun_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_SHOGUN_2));
        let game_selected_napoleon = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_NAPOLEON));
        let game_selected_empire = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_EMPIRE));

        //game_selected_warhammer_3.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_file_name()))).as_ref());
        //game_selected_troy.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_file_name()))).as_ref());
        //game_selected_three_kingdoms.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_file_name()))).as_ref());
        //game_selected_warhammer_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_file_name()))).as_ref());
        //game_selected_warhammer.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_file_name()))).as_ref());
        //game_selected_thrones_of_britannia.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_file_name()))).as_ref());
        //game_selected_attila.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_file_name()))).as_ref());
        //game_selected_rome_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_file_name()))).as_ref());
        //game_selected_shogun_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_file_name()))).as_ref());
        //game_selected_napoleon.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_file_name()))).as_ref());
        //game_selected_empire.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_file_name()))).as_ref());

        let game_selected_group = QActionGroup::new(&menu_bar_game_selected);

        // Configure the `Game Selected` Menu.
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

        //-----------------------------------------------//
        // `About` Menu.
        //-----------------------------------------------//
        let about_about_qt = menu_bar_about.add_action_q_string(&qtr("about_qt"));
        let about_about_runcher = menu_bar_about.add_action_q_string(&qtr("about_runcher"));
        let about_check_updates = menu_bar_about.add_action_q_string(&qtr("check_updates"));

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

            // NOTE: This loads arena on purpose, so ANY game selected triggers a game change properly.
            game_selected: Rc::new(RwLock::new(SUPPORTED_GAMES.game("arena").unwrap().clone())),
        });

        let slots = AppUISlots::new(&app_ui);
        app_ui.set_connections(&slots);

        // Initialize settings.
        init_settings(&app_ui.main_window().static_upcast());

        // Load the correct theme.
        Self::reload_theme();

        // Apply last ui state.
        app_ui.main_window().restore_geometry(&setting_byte_array("geometry"));
        app_ui.main_window().restore_state_1a(&setting_byte_array("windowState"));

        // Show the Main Window.
        app_ui.main_window().show();
        log_to_status_bar(app_ui.main_window().status_bar(), "Initializing, please wait...");

        // If we have it enabled in the prefs, check if there are updates.
        if setting_bool("check_updates_on_start") {
            app_ui.check_updates(false);
        }

        // Set the game selected based on the default game. If we passed a game through an argument, use that one.
        //
        // Note: set_checked does *NOT* trigger the slot for changing game selected. We need to trigger that one manually.
        let mut default_game = setting_string("default_game");
        let args = args().collect::<Vec<String>>();
        if args.len() == 2 {
            match &*args[1] {
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
        app_ui.load_data(&default_game)?;

        Ok(app_ui)
    }

    pub unsafe fn set_connections(&self, slots: &AppUISlots) {
        self.actions_ui().play_button().released().connect(slots.launch_game());
        self.actions_ui().enable_logging().toggled().connect(slots.toggle_logging());
        self.actions_ui().enable_skip_intro().toggled().connect(slots.toggle_skip_intros());
        self.actions_ui().settings_button().released().connect(slots.open_settings());
        self.actions_ui().folders_button().released().connect(slots.open_folders_submenu());
        self.actions_ui().open_game_root_folder().triggered().connect(slots.open_game_root_folder());
        self.actions_ui().open_game_data_folder().triggered().connect(slots.open_game_data_folder());
        self.actions_ui().open_game_content_folder().triggered().connect(slots.open_game_content_folder());
        self.actions_ui().open_runcher_config_folder().triggered().connect(slots.open_runcher_config_folder());
        self.actions_ui().open_runcher_error_folder().triggered().connect(slots.open_runcher_error_folder());
        self.actions_ui().copy_load_order_button().released().connect(slots.copy_load_order());
        self.actions_ui().paste_load_order_button().released().connect(slots.paste_load_order());
        self.actions_ui().reload_button().released().connect(slots.reload());
        self.actions_ui().profile_load_button().released().connect(slots.load_profile());
        self.actions_ui().profile_save_button().released().connect(slots.save_profile());

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
        if reload_same_game || new_game_selected != self.game_selected().read().unwrap().game_key_name() {
            self.load_data(&new_game_selected)?;
        }

        Ok(())
    }

    pub unsafe fn load_data(&self, game: &str) -> Result<()> {

        // We may receive invalid games here, so rule out the invalid ones.
        match SUPPORTED_GAMES.game(game) {
            Some(game) => {
                *self.game_selected().write().unwrap() = game.clone();

                // Load the game's config.
                *self.game_config().write().unwrap() = Some(GameConfig::load(game, true)?);

                // Load the profile's list.
                *self.game_profiles().write().unwrap() = Profile::profiles_for_game(game)?;
                self.actions_ui().profile_model().clear();

                for profile in self.game_profiles().read().unwrap().keys() {
                    self.actions_ui().profile_combobox().add_item_q_string(&QString::from_std_str(profile));
                }

                // If we don't have a path in the settings for the game, disable the play button.
                let game_path_str = setting_string(game.game_key_name());
                self.actions_ui().play_button().set_enabled(!game_path_str.is_empty());

                // If we have a path, load all the mods to the UI.
                let game_path = PathBuf::from(&game_path_str);
                if !game_path_str.is_empty() {
                    let data_paths = game.data_packs_paths(&game_path);
                    let content_paths = game.content_packs_paths(&game_path);

                    let mut steam_ids = vec![];

                    // Initialize the mods in loadable folders.
                    {
                        let mut mods = self.game_config().write().unwrap();
                        if let Some(ref mut mods) = *mods {

                            // Clear the previous paths.
                            mods.mods_mut().values_mut().for_each(|modd| modd.paths_mut().clear());

                            if let Some(ref paths) = data_paths {
                                for path in paths {
                                    let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                                    let pack = Pack::read_and_merge(&[path.to_path_buf()], true, false)?;
                                    if pack.pfh_file_type() == PFHFileType::Mod {
                                        match mods.mods_mut().get_mut(&pack_name) {
                                            Some(modd) => {
                                                if !modd.paths().contains(path) {
                                                    modd.paths_mut().push(path.to_path_buf());
                                                }
                                            }
                                            None => {
                                                let mut modd = Mod::default();
                                                modd.set_name(pack_name.to_owned());
                                                modd.set_id(pack_name.to_owned());
                                                modd.set_paths(vec![path.to_path_buf()]);
                                                mods.mods_mut().insert(pack_name, modd);
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(ref paths) = content_paths {
                                for path in paths {
                                    let pack_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                                    let pack = Pack::read_and_merge(&[path.to_path_buf()], true, false)?;
                                    if pack.pfh_file_type() == PFHFileType::Mod {
                                        match mods.mods_mut().get_mut(&pack_name) {
                                            Some(modd) => {
                                                if !modd.paths().contains(path) {
                                                    modd.paths_mut().push(path.to_path_buf());
                                                }

                                                // Get the steam id from the path, if possible.
                                                let steam_id = path.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();
                                                steam_ids.push(steam_id.to_owned());
                                                modd.set_steam_id(Some(steam_id));

                                            }
                                            None => {
                                                let mut modd = Mod::default();
                                                modd.set_name(pack_name.to_owned());
                                                modd.set_id(pack_name.to_owned());
                                                modd.set_paths(vec![path.to_path_buf()]);

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

                            let _ = populate_mods(mods.mods_mut(), &steam_ids);
                            //if let Err(error) = populate_mods(mods.mods_mut(), &steam_ids) {
                            //    //show_dialog(self.main_window(), error, false);
                            //}
                        }
                    }
                }

                let mods = self.game_config().read().unwrap();
                if let Some(ref mods) = *mods {
                    self.mod_list_ui().load(mods)?;
                    self.pack_list_ui().load(mods, &self.game_selected().read().unwrap(), &game_path)?;
                }

                // Update the launch options for the new game.
                self.actions_ui().enable_logging().set_checked(setting_bool(&format!("enable_logging_{}", game.game_key_name())));
                self.actions_ui().enable_skip_intro().set_checked(setting_bool(&format!("enable_skip_intros_{}", game.game_key_name())));

                Ok(())
            },
            None => Err(anyhow!("Game {} is not a valid game.", game)),
        }
    }

    pub unsafe fn open_settings(&self) {
        let game_selected = self.game_selected().read().unwrap();
        let game_key = game_selected.game_key_name();
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

        if self.actions_ui().enable_logging().is_checked() || self.actions_ui().enable_skip_intro().is_checked() {
            let temp_path_folder = tempfile::tempdir()?.into_path();
            let temp_path_file_name = format!("{}_{}.pack", RESERVED_PACK_NAME, self.game_selected().read().unwrap().game_key_name());
            let temp_path = temp_path_folder.join(&temp_path_file_name);
            folder_list.push_str(&format!("add_working_directory \"{}\";\n", temp_path_folder.to_string_lossy()));
            pack_list.push_str(&format!("mod \"{}\";\n", temp_path_file_name));

            // Generate the reserved pack.
            let pack_version = game.pfh_version_by_file_type(PFHFileType::Mod);
            let mut reserved_pack = Pack::new_with_version(pack_version);

            // Skip videos.
            if self.actions_ui().enable_skip_intro().is_checked() {
                for (game_key, paths) in INTRO_MOVIE_PATHS_BY_GAME {
                    if game.game_key_name() == game_key {
                        for path in paths {
                            let file = RFile::new_from_vec(&EMPTY_MOVIE, FileType::Video, 0, path);
                            reserved_pack.files_mut().insert(path.to_string(), file);
                        }
                        break;
                    }
                }
            }

            // Logging.
            if self.actions_ui().enable_logging().is_checked() {
                let file = RFile::new_from_vec(&[], FileType::Unknown, 0, SCRIPT_DEBUG_ACTIVATOR_PATH);
                reserved_pack.files_mut().insert(SCRIPT_DEBUG_ACTIVATOR_PATH.to_string(), file);
            }

            reserved_pack.save(Some(&temp_path), &game, &None)?;
        }

        pack_list.push_str(&(0..self.pack_list_ui().model().row_count_0a())
            .map(|index| {
                let mut string = String::new();
                let item = self.pack_list_ui().model().item_1a(index);
                let item_path = self.pack_list_ui().model().item_2a(index, 1);
                let item_location = self.pack_list_ui().model().item_2a(index, 3);
                let item_steam_id = self.pack_list_ui().model().item_2a(index, 4);

                let steam_id = item_steam_id.text().to_std_string();
                if item_location.text().to_std_string().starts_with("Content") && !steam_id.is_empty() {
                    let mut path = PathBuf::from(item_path.text().to_std_string());
                    path.pop();

                    let mut path_str = path.to_string_lossy().to_string();
                    if path_str.starts_with("\\\\?\\") {
                        path_str = path_str[4..].to_string();
                    }
                    folder_list.push_str(&format!("add_working_directory \"{}\";\n", path_str));
                }
                string.push_str(&format!("mod \"{}\";", item.text().to_std_string()));
                string
            })
            .collect::<Vec<_>>()
            .join("\n"));

        let game_path = setting_path(game.game_key_name());
        let file_path = game_path.join("mod_list.txt");

        let mut file = BufWriter::new(File::create(file_path)?);
        file.write_all(folder_list.as_bytes())?;
        file.write_all(pack_list.as_bytes())?;
        file.flush()?;

        let exec_game = game.executable_path(&game_path).unwrap();

        if cfg!(target_os = "windows") {
            let mut command = SystemCommand::new("cmd");
            command.arg("/C");
            command.arg("start");
            command.arg("/d");
            command.arg(game_path.to_string_lossy().replace('\\', "/"));
            command.arg(exec_game.file_name().unwrap().to_string_lossy().to_string());
            command.arg("mod_list.txt;");

            // This disables the terminal when executing the command.
            #[cfg(target_os = "windows")]command.creation_flags(CREATE_NO_WINDOW);
            command.spawn()?;

            Ok(())
        } else if cfg!(target_os = "linux") {
            Err(anyhow!("Unsupported OS."))
        } else {
            Err(anyhow!("Unsupported OS."))
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

    /// This function creates the stylesheet used for the dark theme in windows.
    pub fn dark_stylesheet() -> Result<String> {
        let mut file = File::open(ASSETS_PATH.join("dark-theme.qss"))?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        Ok(string.replace("{assets_path}", &ASSETS_PATH.to_string_lossy().replace("\\", "/")))
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
            let game_path = setting_path(game.game_key_name());
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
