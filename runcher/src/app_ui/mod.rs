//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
use qt_widgets::QButtonGroup;
use qt_widgets::QComboBox;
use qt_widgets::QGroupBox;
use qt_widgets::QLineEdit;
use qt_widgets::QRadioButton;
use qt_widgets::QTabWidget;
use qt_widgets::QToolBar;
use qt_widgets::{QDialog, QDialogButtonBox, q_dialog_button_box::StandardButton};
use qt_widgets::QLabel;
use qt_widgets::QMainWindow;
use qt_widgets::QMessageBox;
use qt_widgets::q_message_box;
use qt_widgets::QPushButton;
use qt_widgets::QSplitter;
use qt_widgets::QTableView;
use qt_widgets::QTextEdit;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QFont;
use qt_gui::QIcon;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CheckState;
use qt_core::Orientation;
use qt_core::QBox;
use qt_core::QCoreApplication;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QSize;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::SlotNoArgs;

use cpp_core::CppBox;
use cpp_core::Ref;

use anyhow::{anyhow, Result};
use base64::prelude::*;
use crossbeam::channel::Receiver;
use flate2::read::ZlibDecoder;
use getset::Getters;
use itertools::Itertools;
use rayon::prelude::*;
use sha256::try_digest;

use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use rpfm_lib::binary::{ReadBytes, WriteBytes};
use rpfm_lib::files::{Container, db::DB, EncodeableExtraData, FileType, loc::Loc, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType, supported_games::*};
use rpfm_lib::integrations::log::*;
use rpfm_lib::schema::Schema;
use rpfm_lib::utils::{files_from_subdir, path_to_absolute_string};

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::clone;
use rpfm_ui_common::locale::*;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::tools::*;
use rpfm_ui_common::utils::*;

use crate::actions_ui::ActionsUI;
use crate::CENTRAL_COMMAND;
use crate::cli::Cli;
use crate::communications::*;
use crate::DARK_PALETTE;
use crate::data_ui::DataListUI;
use crate::data_ui::pack_tree::PackTree;
use crate::ffi::*;
use crate::games::*;
use crate::mod_manager::{*, game_config::{GameConfig, DEFAULT_CATEGORY}, integrations::*, load_order::{ImportedLoadOrderMode, LoadOrder}, mods::{Mod, ShareableMod}, profiles::Profile, saves::Save};
use crate::LIGHT_PALETTE;
use crate::LIGHT_STYLE_SHEET;
use crate::mod_list_ui::*;
use crate::pack_list_ui::PackListUI;
use crate::{
    REGEX_MAP_INFO_DISPLAY_NAME,
    REGEX_MAP_INFO_DESCRIPTION,
    REGEX_MAP_INFO_TYPE,
    REGEX_MAP_INFO_TEAM_SIZE_1,
    REGEX_MAP_INFO_TEAM_SIZE_2,
    REGEX_MAP_INFO_DEFENDER_FUNDS_RATIO,
    REGEX_MAP_INFO_HAS_KEY_BUILDINGS
};
use crate::SCHEMA;
use crate::settings_ui::*;
use crate::SUPPORTED_GAMES;
use crate::updater_ui::*;

use self::slots::AppUISlots;

pub mod slots;

const LOAD_ORDER_STRING_VIEW_DEBUG: &str = "ui_templates/load_order_string_dialog.ui";
const LOAD_ORDER_STRING_VIEW_RELEASE: &str = "ui/load_order_string_dialog.ui";

const WORKSHOP_UPLOAD_VIEW_DEBUG: &str = "ui_templates/workshop_upload_dialog.ui";
const WORKSHOP_UPLOAD_VIEW_RELEASE: &str = "ui/workshop_upload_dialog.ui";

const LOG_ANALYSIS_VIEW_DEBUG: &str = "ui_templates/log_analysis_dialog.ui";
const LOG_ANALYSIS_VIEW_RELEASE: &str = "ui/log_analysis_dialog.ui";

const MERGE_ALL_PACKS_PACK_NAME: &str = "merge_me_sideways_honey";

#[allow(dead_code)] const VANILLA_MOD_LIST_FILE_NAME: &str = "used_mods.txt";
#[allow(dead_code)] pub const CUSTOM_MOD_LIST_FILE_NAME: &str = "mod_list.txt";
#[allow(dead_code)] const USER_SCRIPT_FILE_NAME: &str = "user.script.txt";
#[allow(dead_code)] const USER_SCRIPT_EMPIRE_FILE_NAME: &str = "user.empire_script.txt";

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
    right_tabbar: QBox<QTabWidget>,

    github_button: QBox<QPushButton>,
    discord_button: QBox<QPushButton>,
    patreon_button: QBox<QPushButton>,
    about_runcher_button: QBox<QPushButton>,
    check_updates_button: QBox<QPushButton>,

    //-------------------------------------------------------------------------------//
    // `Game Selected` menu.
    //-------------------------------------------------------------------------------//
    game_selected_pharaoh_dynasties: QPtr<QAction>,
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
    // `Actions` section.
    //-------------------------------------------------------------------------------//
    actions_ui: Rc<ActionsUI>,

    //-------------------------------------------------------------------------------//
    // `Mod List` section.
    //-------------------------------------------------------------------------------//
    mod_list_ui: Rc<ModListUI>,

    //-------------------------------------------------------------------------------//
    // `Data List` section.
    //-------------------------------------------------------------------------------//
    data_list_ui: Rc<DataListUI>,

    //-------------------------------------------------------------------------------//
    // `Pack List` section.
    //-------------------------------------------------------------------------------//
    pack_list_ui: Rc<PackListUI>,

    //-------------------------------------------------------------------------------//
    // Extra stuff
    //-------------------------------------------------------------------------------//
    slots: Rc<RwLock<Option<AppUISlots>>>,
    focused_widget: Rc<RwLock<Option<QPtr<QWidget>>>>,
    disabled_counter: Rc<RwLock<u32>>,

    tools: Arc<RwLock<Tools>>,
    game_config: Arc<RwLock<Option<GameConfig>>>,
    game_load_order: Arc<RwLock<LoadOrder>>,
    game_profiles: Arc<RwLock<HashMap<String, Profile>>>,
    game_saves: Arc<RwLock<Vec<Save>>>,

    // Game selected. Unlike RPFM, here it's not a global.
    game_selected: Rc<RwLock<GameInfo>>,
}

#[derive(Debug, Default, Getters)]
#[getset(get = "pub")]
pub struct ScriptBreak {
    posible_pack: String,
    posible_pack_mod: String,
    posible_pack_link: Option<String>,
    full_log: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AppUI {

    /// This function creates an entire `AppUI` struct. Used to create the entire UI at start.
    pub unsafe fn new() -> Result<Rc<Self>> {

        // Initialize and configure the main window.
        let main_window = launcher_window_safe(setting_bool("dark_mode"));
        let central_widget = QWidget::new_1a(&main_window);
        let central_layout = create_grid_layout(central_widget.static_upcast());
        main_window.set_central_widget(&central_widget);
        main_window.resize_2a(1300, 1100);
        main_window.set_window_title(&QString::from_std_str("The Runcher"));
        QApplication::set_window_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/runcher.png", ASSETS_PATH.to_string_lossy()))));

        let splitter = QSplitter::from_q_widget(&central_widget);
        let left_widget = QWidget::new_1a(&splitter);
        let right_widget = QWidget::new_1a(&splitter);
        let _ = create_grid_layout(left_widget.static_upcast());
        let right_layout = create_grid_layout(right_widget.static_upcast());
        splitter.set_stretch_factor(0, 1);
        right_widget.set_minimum_width(540);

        // Right layout has a tabbar on the second item.
        let right_tabbar = QTabWidget::new_1a(&right_widget);
        right_layout.add_widget_5a(&right_tabbar, 1, 0, 1, 1);

        central_layout.add_widget_5a(splitter.into_raw_ptr(), 0, 1, 1, 1);

        // Get the Status bar.
        let status_bar = main_window.status_bar();
        status_bar.set_size_grip_enabled(false);

        let github_button = QPushButton::from_q_widget(&status_bar);
        github_button.set_flat(true);
        github_button.set_tool_tip(&qtr("github_link"));
        github_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github.svg", ASSETS_PATH.to_string_lossy()))));
        status_bar.add_permanent_widget_1a(&github_button);

        let discord_button = QPushButton::from_q_widget(&status_bar);
        discord_button.set_flat(true);
        discord_button.set_tool_tip(&qtr("discord_link"));
        discord_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/discord.svg", ASSETS_PATH.to_string_lossy()))));
        status_bar.add_permanent_widget_1a(&discord_button);

        let patreon_button = QPushButton::from_q_widget(&status_bar);
        patreon_button.set_flat(true);
        patreon_button.set_tool_tip(&qtr("patreon_link"));
        patreon_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/patreon.png", ASSETS_PATH.to_string_lossy()))));
        status_bar.add_permanent_widget_1a(&patreon_button);

        let about_runcher_button = QPushButton::from_q_widget(&status_bar);
        about_runcher_button.set_flat(true);
        about_runcher_button.set_tool_tip(&qtr("about_runcher"));
        about_runcher_button.set_icon(&QIcon::from_theme_1a(&QString::from_std_str("help-about-symbolic")));
        status_bar.add_permanent_widget_1a(&about_runcher_button);

        let check_updates_button = QPushButton::from_q_widget(&status_bar);
        check_updates_button.set_flat(true);
        check_updates_button.set_tool_tip(&qtr("check_updates"));
        check_updates_button.set_icon(&QIcon::from_theme_1a(&QString::from_std_str("svn-update")));
        status_bar.add_permanent_widget_1a(&check_updates_button);

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
        let game_selected_pharaoh_dynasties = game_selected_bar.add_action_2a(&QIcon::from_q_string(&QString::from_std_str(icon_folder.clone() + SUPPORTED_GAMES.game(KEY_PHARAOH_DYNASTIES).unwrap().icon_small())), &QString::from_std_str(DISPLAY_NAME_PHARAOH_DYNASTIES));
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
        game_selected_group.add_action_q_action(&game_selected_pharaoh_dynasties);
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
        game_selected_pharaoh_dynasties.set_checkable(true);
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
        let data_list_ui = DataListUI::new(&right_tabbar)?;

        //-------------------------------------------------------------------------------//
        // `Pack List` section.
        //-------------------------------------------------------------------------------//
        let pack_list_ui = PackListUI::new(&right_tabbar)?;

        let app_ui = Rc::new(Self {

            //-------------------------------------------------------------------------------//
            // Main Window.
            //-------------------------------------------------------------------------------//
            main_window,
            right_tabbar,

            github_button,
            discord_button,
            patreon_button,
            about_runcher_button,
            check_updates_button,

            //-------------------------------------------------------------------------------//
            // "Game Selected" menu.
            //-------------------------------------------------------------------------------//
            game_selected_pharaoh_dynasties,
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
            // `Actions` section.
            //-------------------------------------------------------------------------------//
            actions_ui,

            //-------------------------------------------------------------------------------//
            // `Mod List` section.
            //-------------------------------------------------------------------------------//
            mod_list_ui,

            //-------------------------------------------------------------------------------//
            // `Data List` section.
            //-------------------------------------------------------------------------------//
            data_list_ui,

            //-------------------------------------------------------------------------------//
            // `Pack List` section.
            //-------------------------------------------------------------------------------//
            pack_list_ui,

            //-------------------------------------------------------------------------------//
            // "Extra stuff" menu.
            //-------------------------------------------------------------------------------//
            slots: Rc::new(RwLock::new(None)),
            focused_widget: Rc::new(RwLock::new(None)),
            disabled_counter: Rc::new(RwLock::new(0)),

            tools: Arc::new(RwLock::new(Tools::load(&None).unwrap_or_else(|_| Tools::default()))),
            game_config: Arc::new(RwLock::new(None)),
            game_load_order: Arc::new(RwLock::new(LoadOrder::default())),
            game_profiles: Arc::new(RwLock::new(HashMap::new())),
            game_saves: Arc::new(RwLock::new(vec![])),

            // NOTE: This loads arena on purpose, so ANY game selected triggers a game change properly.
            game_selected: Rc::new(RwLock::new(SUPPORTED_GAMES.game("arena").unwrap().clone())),
        });

        let slots = AppUISlots::new(&app_ui);
        app_ui.set_connections(&slots);
        *app_ui.slots.write().unwrap() = Some(slots);

        // Initialize settings.
        init_settings(&app_ui.main_window().static_upcast());

        // Disable the games we don't have a path for (uninstalled) and Shogun 2, as it's not supported yet.
        for game in SUPPORTED_GAMES.games_sorted().iter() {
            let has_exe = game.executable_path(&setting_path(game.key())).filter(|path| path.is_file()).is_some();
            match game.key() {
                KEY_PHARAOH_DYNASTIES => {
                    app_ui.game_selected_pharaoh_dynasties().set_enabled(has_exe);
                    app_ui.game_selected_pharaoh_dynasties().set_visible(has_exe);
                }
                KEY_PHARAOH => {
                    app_ui.game_selected_pharaoh().set_enabled(has_exe);
                    app_ui.game_selected_pharaoh().set_visible(has_exe);
                }
                KEY_WARHAMMER_3 => {
                    app_ui.game_selected_warhammer_3().set_enabled(has_exe);
                    app_ui.game_selected_warhammer_3().set_visible(has_exe);
                }
                KEY_TROY => {
                    app_ui.game_selected_troy().set_enabled(has_exe);
                    app_ui.game_selected_troy().set_visible(has_exe);
                }
                KEY_THREE_KINGDOMS => {
                    app_ui.game_selected_three_kingdoms().set_enabled(has_exe);
                    app_ui.game_selected_three_kingdoms().set_visible(has_exe);
                }
                KEY_WARHAMMER_2 => {
                    app_ui.game_selected_warhammer_2().set_enabled(has_exe);
                    app_ui.game_selected_warhammer_2().set_visible(has_exe);
                }
                KEY_WARHAMMER => {
                    app_ui.game_selected_warhammer().set_enabled(has_exe);
                    app_ui.game_selected_warhammer().set_visible(has_exe);
                }
                KEY_THRONES_OF_BRITANNIA => {
                    app_ui.game_selected_thrones_of_britannia().set_enabled(has_exe);
                    app_ui.game_selected_thrones_of_britannia().set_visible(has_exe);
                }
                KEY_ATTILA => {
                    app_ui.game_selected_attila().set_enabled(has_exe);
                    app_ui.game_selected_attila().set_visible(has_exe);
                }
                KEY_ROME_2 => {
                    app_ui.game_selected_rome_2().set_enabled(has_exe);
                    app_ui.game_selected_rome_2().set_visible(has_exe);
                }
                KEY_SHOGUN_2 => {
                    app_ui.game_selected_shogun_2().set_enabled(has_exe);
                    app_ui.game_selected_shogun_2().set_visible(has_exe);
                }
                KEY_NAPOLEON => {
                    app_ui.game_selected_napoleon().set_enabled(has_exe);
                    app_ui.game_selected_napoleon().set_visible(has_exe);
                }
                KEY_EMPIRE => {
                    app_ui.game_selected_empire().set_enabled(has_exe);
                    app_ui.game_selected_empire().set_visible(has_exe);
                }
                _ => {},
            }
        }

        // Load the correct theme.
        app_ui.reload_theme();

        // Apply last ui state.
        app_ui.main_window().restore_geometry(&setting_byte_array("geometry"));
        app_ui.main_window().restore_state_1a(&setting_byte_array("windowState"));

        // Default the right tabs to the pack list.
        app_ui.right_tabbar().set_current_index(1);

        // Apply the font.
        let font_name = setting_string("font_name");
        let font_size = setting_int("font_size");
        let font = QFont::from_q_string_int(&QString::from_std_str(font_name), font_size);
        QApplication::set_font_1a(&font);

        // Initialization logic. This takes care of parsing args for stuff like profile shortcuts,
        // or setting the game selected.
        //
        // NOTE: This exits if autostart param is passed, or if you pass invalid params,
        // so we don't need to load anything regarthing the UI.
        match Cli::parse_args(&app_ui) {
            Ok((autostart, network_receiver)) => if autostart {
                exit(0);
            } else {

                // Ignore network errors.
                let _ = app_ui.update_mod_list_with_online_data(&network_receiver);
            },

            // Do not close on incorrect args.
            Err(error) => show_dialog(app_ui.main_window(), error, false),
        }

        // Check for updates.
        UpdaterUI::new_with_precheck(&app_ui)?;

        Ok(app_ui)
    }

    pub unsafe fn set_connections(&self, slots: &AppUISlots) {
        self.actions_ui().play_button().released().connect(slots.launch_game());
        self.actions_ui().enable_logging_checkbox().toggled().connect(slots.toggle_logging());
        self.actions_ui().enable_skip_intro_checkbox().toggled().connect(slots.toggle_skip_intros());
        self.actions_ui().remove_trait_limit_checkbox().toggled().connect(slots.toggle_remove_trait_limit());
        self.actions_ui().remove_siege_attacker_checkbox().toggled().connect(slots.toggle_remove_siege_attacker());
        self.actions_ui().merge_all_mods_checkbox().toggled().connect(slots.toggle_merge_all_mods());
        self.actions_ui().enable_translations_combobox().current_text_changed().connect(slots.toggle_enable_translations());
        self.actions_ui().unit_multiplier_spinbox().value_changed().connect(slots.change_unit_multiplier());
        self.actions_ui().settings_button().released().connect(slots.open_settings());
        self.actions_ui().universal_rebalancer_combobox().current_text_changed().connect(slots.toggle_universal_rebalancer());
        self.actions_ui().enable_dev_only_ui_checkbox().toggled().connect(slots.toggle_dev_only_ui());
        self.actions_ui().folders_button().released().connect(slots.open_folders_submenu());
        self.actions_ui().open_game_root_folder().triggered().connect(slots.open_game_root_folder());
        self.actions_ui().open_game_data_folder().triggered().connect(slots.open_game_data_folder());
        self.actions_ui().open_game_content_folder().triggered().connect(slots.open_game_content_folder());
        self.actions_ui().open_game_secondary_folder().triggered().connect(slots.open_game_secondary_folder());
        self.actions_ui().open_game_config_folder().triggered().connect(slots.open_game_config_folder());
        self.actions_ui().open_runcher_config_folder().triggered().connect(slots.open_runcher_config_folder());
        self.actions_ui().open_runcher_error_folder().triggered().connect(slots.open_runcher_error_folder());
        self.actions_ui().copy_load_order_button().released().connect(slots.copy_load_order());
        self.actions_ui().paste_load_order_button().released().connect(slots.paste_load_order());
        self.actions_ui().reload_button().released().connect(slots.reload());
        self.actions_ui().download_subscribed_mods_button().released().connect(slots.download_subscribed_mods());
        self.actions_ui().profile_load_button().released().connect(slots.load_profile());
        self.actions_ui().profile_save_button().released().connect(slots.save_profile());
        self.actions_ui().profile_manager_button().released().connect(slots.open_profile_manager());

        self.game_selected_pharaoh_dynasties().triggered().connect(slots.change_game_selected());
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

        self.about_runcher_button().released().connect(slots.about_runcher());
        self.check_updates_button().released().connect(slots.check_updates());

        self.github_button().released().connect(slots.github_link());
        self.discord_button().released().connect(slots.discord_link());
        self.patreon_button().released().connect(slots.patreon_link());

        self.mod_list_ui().model().item_changed().connect(slots.update_pack_list());
        self.mod_list_ui().upload_to_workshop().triggered().connect(slots.upload_to_workshop());
        self.mod_list_ui().download_from_workshop().triggered().connect(slots.download_from_workshop());
        self.mod_list_ui().context_menu().about_to_show().connect(slots.mod_list_context_menu_open());
        self.mod_list_ui().enable_selected().triggered().connect(slots.enable_selected());
        self.mod_list_ui().disable_selected().triggered().connect(slots.disable_selected());
        self.mod_list_ui().category_new().triggered().connect(slots.category_create());
        self.mod_list_ui().category_delete().triggered().connect(slots.category_delete());
        self.mod_list_ui().category_rename().triggered().connect(slots.category_rename());
        self.mod_list_ui().category_sort().triggered().connect(slots.category_sort());
        draggable_tree_view_drop_signal(self.mod_list_ui().tree_view().static_upcast()).connect(slots.category_move());

        self.mod_list_ui().copy_to_secondary().triggered().connect(slots.copy_to_secondary());
        self.mod_list_ui().move_to_secondary().triggered().connect(slots.move_to_secondary());

        self.pack_list_ui().automatic_order_button().toggled().connect(slots.pack_toggle_auto_sorting());
        draggable_tree_view_drop_signal(self.pack_list_ui().tree_view().static_upcast()).connect(slots.pack_move());

        self.data_list_ui().reload_button().released().connect(slots.data_view_reload());
        self.data_list_ui().tree_view().double_clicked().connect(slots.open_file_with_rpfm());
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

    pub unsafe fn change_game_selected(&self, reload_same_game: bool, skip_network_update: bool) -> Result<Option<Receiver<Response>>> {

        // Get the new `Game Selected` and clean his name up, so it ends up like "x_y".
        let mut new_game_selected = self.game_selected_group.checked_action().text().to_std_string();
        if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
        let new_game_selected = new_game_selected.replace(' ', "_").to_lowercase();

        // If the game changed or we're initializing the program, change the game selected.
        //
        // This works because by default, the initially stored game selected is arena, and that one can never set manually.
        if reload_same_game || new_game_selected != self.game_selected().read().unwrap().key() {
            self.toggle_main_window(false);

            let event_loop = qt_core::QEventLoop::new_0a();
            event_loop.process_events_0a();

            let result = self.load_data(&new_game_selected, skip_network_update);

            self.toggle_main_window(true);
            result
        } else {
            Ok(None)
        }
    }

    pub unsafe fn load_data(&self, game: &str, skip_network_update: bool) -> Result<Option<Receiver<Response>>> {

        // We may receive invalid games here, so rule out the invalid ones.
        match SUPPORTED_GAMES.game(game) {
            Some(game) => {

                // Schemas are optional, so don't interrupt loading due to they not being present.
                let schema_path = schemas_path().unwrap().join(game.schema_file_name());
                *SCHEMA.write().unwrap() = Schema::load(&schema_path, None).ok();
                *self.game_selected().write().unwrap() = game.clone();

                // Trigger an update of all game configs, just in case one needs update.
                let _ = GameConfig::update(game.key());

                // Load the game's config and last known load order.
                *self.game_load_order().write().unwrap() = LoadOrder::load(game).unwrap_or_else(|_| Default::default());
                *self.game_config().write().unwrap() = Some(GameConfig::load(game, true)?);

                // Trigger an update of all game profiles, just in case one needs update.
                let _ = Profile::update(&self.game_config().read().unwrap().clone().unwrap(), game);

                // Load the profile's list.
                match Profile::profiles_for_game(game) {
                    Ok(profiles) => *self.game_profiles().write().unwrap() = profiles,
                    Err(error) => show_dialog(self.main_window(), format!("Error loading profiles: {}", error), false),
                }

                self.actions_ui().profile_model().clear();
                for profile in self.game_profiles().read().unwrap().keys().sorted() {
                    self.actions_ui().profile_combobox().add_item_q_string(&QString::from_std_str(profile));
                }

                // Load the saves list for the selected game.
                let game_path_str = setting_string(game.key());
                let game_path = PathBuf::from(&game_path_str);
                if let Err(error) = self.load_saves_to_ui(game, &game_path) {
                    show_dialog(self.main_window(), error, false);
                }

                // Load the mods to the UI. This does an early return, just in case you add something after this.
                match self.load_mods_to_ui(game, &game_path, skip_network_update) {
                    Ok(network_receiver) => {

                        // Load the launch options for the game selected, as some of them may depend on mods we just loaded.
                        let _ = setup_actions(self, game, self.game_config().read().unwrap().as_ref().unwrap(), &game_path, &self.game_load_order().read().unwrap());

                        return Ok(network_receiver)
                    },
                    Err(error) => show_dialog(self.main_window(), error, false),
                }

                Ok(None)
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

                    let mut save = Save::default();
                    save.set_path(save_path.to_path_buf());
                    save.set_name(save_path.file_name().unwrap().to_string_lossy().to_string());

                    /*
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


                    }*/
                    let item = QStandardItem::from_q_string(&QString::from_std_str(save.name()));
                    self.actions_ui().save_model().append_row_q_standard_item(item.into_ptr());

                    game_saves.push(save);
                }
            }
        }

        Ok(())
    }

    pub unsafe fn load_mods_to_ui(&self, game: &GameInfo, game_path: &Path, skip_network_update: bool) -> Result<Option<Receiver<Response>>> {
        let mut mods = self.game_config().write().unwrap();
        if let Some(ref mut mods) = *mods {
            let mut load_order = self.game_load_order().write().unwrap();
            let network_receiver = mods.update_mod_list(game, game_path, &mut load_order, skip_network_update)?;

            self.mod_list_ui().load(game, mods)?;
            self.pack_list_ui().load(mods, game, game_path, &load_order)?;

            Ok(network_receiver)
        } else {
            Ok(None)
        }
    }

    pub unsafe fn open_settings(&self) {
        let game_key = self.game_selected().read().unwrap().key().to_owned();
        let game_path_old = setting_path(&game_key);
        let dark_theme_old = setting_bool("dark_mode");
        let font_name_old = setting_string("font_name");
        let font_size_old = setting_int("font_size");

        match SettingsUI::new(self.main_window()) {
            Ok(saved) => {
                if saved {
                    let game_path_new = setting_path(&game_key);

                    // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                    // re-select the current `GameSelected` to force it to reload the game's files.
                    if game_path_old != game_path_new {
                        QAction::trigger(&self.game_selected_group.checked_action());
                    }

                    // Reload the tools, just in case they changed.
                    *self.tools().write().unwrap() = Tools::load(&None).unwrap_or_else(|_| Tools::default());

                    // Disable the games we don't have a path for (uninstalled).
                    for game in SUPPORTED_GAMES.games_sorted().iter() {
                        let has_exe = game.executable_path(&setting_path(game.key())).filter(|path| path.is_file()).is_some();
                        match game.key() {
                            KEY_PHARAOH_DYNASTIES => self.game_selected_pharaoh_dynasties().set_enabled(has_exe),
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

                    // If we detect a change in theme, reload it.
                    let dark_theme_new = setting_bool("dark_mode");
                    if dark_theme_old != dark_theme_new {
                        self.reload_theme();
                    }

                    // If we detect a change in the saved font, trigger a font change.
                    let font_name = setting_string("font_name");
                    let font_size = setting_int("font_size");
                    if font_name_old != font_name || font_size_old != font_size {
                        let font = QFont::from_q_string_int(&QString::from_std_str(&font_name), font_size);
                        QApplication::set_font_1a(&font);
                    }

                    // If we detect a factory reset, reset the window's geometry and state.
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
        let game = self.game_selected().read().unwrap().clone();
        let game_path = setting_path(game.key());
        let data_path = game.data_path(&game_path)?;

        // If we have "merge all mods" checked, we need to load the entire load order into a single pack, and load that pack instead of the entire load order.
        //
        // TODO: Review this before re-enabling merged mods. This pretty sure breaks on older games.
        if self.actions_ui().merge_all_mods_checkbox().is_enabled() && self.actions_ui().merge_all_mods_checkbox().is_checked() {
            let temp_path_file_name = format!("{}_{}.pack", MERGE_ALL_PACKS_PACK_NAME, self.game_selected().read().unwrap().key());
            let temp_path = data_path.join(&temp_path_file_name);
            pack_list.push_str(&format!("mod \"{}\";", temp_path_file_name));

            // Generate the merged pack.
            let load_order = self.game_load_order().read().unwrap();
            if let Some(ref game_config) = *self.game_config().read().unwrap() {

                let pack_paths = load_order.mods().iter()
                    .filter_map(|mod_id| {
                        let modd = game_config.mods().get(mod_id)?;
                        std::fs::canonicalize(modd.paths().first()?).ok()
                    })
                .collect::<Vec<_>>();

                if !pack_paths.is_empty() {
                    let mut reserved_pack = Pack::read_and_merge(&pack_paths, &game, true, false, true)?;
                    let pack_version = game.pfh_version_by_file_type(PFHFileType::Mod);
                    reserved_pack.set_pfh_version(pack_version);

                    let mut encode_data = EncodeableExtraData::default();
                    encode_data.set_nullify_dates(true);
                    encode_data.set_game_info(Some(&game));

                    reserved_pack.save(Some(&temp_path), &game, &Some(encode_data))?;
                }
            } else {
                return Err(anyhow!(tr("game_config_error")));
            }
        }

        // Otherwise, just add the packs from the load order to the text file.
        else if let Some(ref game_config) = *self.game_config().read().unwrap() {
            let load_order = self.game_load_order().read().unwrap();
            load_order.build_load_order_string(game_config, &game, &data_path, &mut pack_list, &mut folder_list);
        }

        // If our folder list contains the secondary folder, we need to make sure we create the masks folder in it,
        // and mask in there all non-enabled movie files. Note that we only use this in games older than warhammer. Newer games use the exclude_pack_file command.
        if *game.raw_db_version() <= 1 || (*game.raw_db_version() == 2 && (game.key() == KEY_ROME_2 || game.key() == KEY_ATTILA || game.key() == KEY_THRONES_OF_BRITANNIA)) {
            let secondary_mods_path = secondary_mods_path(game.key()).unwrap_or_else(|_| PathBuf::new());
            let secondary_mods_path_str = path_to_absolute_string(&secondary_mods_path);

            if secondary_mods_path.is_dir() && folder_list.contains(&secondary_mods_path_str) {
                let masks_path = secondary_mods_path.join(SECONDARY_FOLDER_NAME);

                // Remove all files in it so previous maskings do not interfere.
                if masks_path.is_dir() {
                    std::fs::remove_dir_all(&masks_path)?;
                }

                DirBuilder::new().recursive(true).create(&masks_path)?;

                let mut mask_pack = Pack::new_with_version(game.pfh_version_by_file_type(PFHFileType::Movie));
                mask_pack.set_pfh_file_type(PFHFileType::Movie);

                if let Some(ref game_config) = *self.game_config().read().unwrap() {
                    for path in std::fs::read_dir(secondary_mods_path)? {
                        let file_name = path?.file_name().to_string_lossy().to_string();

                        if let Some(modd) = game_config.mods().get(&file_name) {
                            if modd.pack_type() == &PFHFileType::Movie && !modd.enabled(&game, &data_path) {
                                mask_pack.save(Some(&masks_path.join(file_name)), &game, &None)?;
                            }
                        }
                    }
                }
            }
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

            // Empire has its own user script.
            if game.key() == KEY_EMPIRE {
                scripts_path.join(USER_SCRIPT_EMPIRE_FILE_NAME)
            } else {
                scripts_path.join(USER_SCRIPT_FILE_NAME)
            }
        };

        // Setup the launch options stuff. This may add a line to the folder list, so we need to resave the load order file after this.
        let folder_list_pre = folder_list.to_owned();
        Self::save_load_order_file(&file_path, &game, &folder_list, &pack_list)?;
        prepare_launch_options(self, &game, &data_path, &mut folder_list)?;

        if folder_list != folder_list_pre {
            Self::save_load_order_file(&file_path, &game, &folder_list, &pack_list)?;
        }

        // Launch is done through workshopper to getup the Steam Api.
        //
        // Here we just build the commands and pass them to workshopper.
        match game.executable_path(&game_path) {
            Some(exec_game) => {
                if cfg!(target_os = "windows") {

                    // For post-shogun 2 games, we use the same command to bypass the launcher.
                    let command = if *game.raw_db_version() >= 1 {
                        let mut command = format!("cmd /C start /W /d \"{}\" \"{}\" {};",
                            game_path.to_string_lossy().replace('\\', "/"),
                            exec_game.file_name().unwrap().to_string_lossy(),
                            CUSTOM_MOD_LIST_FILE_NAME
                        );

                        for arg in &extra_args {
                            command.push(' ');
                            command.push_str(arg);
                        }

                        command
                    }

                    // Empire and Napoleon do not have a launcher. We can make our lives easier calling steam instead of launching the game manually.
                    else {
                        format!("cmd /C start /W /d \"{}\" \"{}\" \"{}\";",
                            game_path.to_string_lossy().replace('\\', "/"),
                            exec_game.file_name().unwrap().to_string_lossy(),
                            file_path.to_string_lossy().replace('\\', "/")
                        )
                    };

                    self.toggle_main_window(false);

                    let event_loop = qt_core::QEventLoop::new_0a();
                    event_loop.process_events_0a();

                    let start_date = SystemTime::now();
                    let command = BASE64_STANDARD.encode(command);

                    let wait_for_finish = setting_bool("check_logs");
                    let result = crate::mod_manager::integrations::launch_game(&game, &command, wait_for_finish);

                    // Check the logs post-launch, if there's any log to check.
                    if setting_bool("check_logs") {
                        self.check_logs(&game, &game_path, &start_date)?;
                    }

                    self.toggle_main_window(true);

                    result
                } else if cfg!(target_os = "linux") {
                    Err(anyhow!("Unsupported OS."))
                } else {
                    Err(anyhow!("Unsupported OS."))
                }
            }
            None => Err(anyhow!("Executable path not found. Is the game folder configured correctly in the settings?"))
        }
    }

    pub unsafe fn load_profile(&self, profile_name: Option<String>, is_autostart: bool) -> Result<()> {
        let profile_name = if let Some(profile_name) = profile_name {
            profile_name
        } else {
            self.actions_ui().profile_combobox().current_text().to_std_string()
        };

        if profile_name.is_empty() {
            return Err(anyhow!("Profile name is empty."));
        }

        match self.game_profiles().read().unwrap().get(&profile_name) {
            Some(profile) => {

                // First, disable all mods, so we return to a neutral state.
                self.mod_list_ui().model().block_signals(true);

                for cat in 0..self.mod_list_ui().model().row_count_0a() {
                    let category = self.mod_list_ui().model().item_1a(cat);
                    for row in 0..category.row_count() {
                        let item = category.child_1a(row);
                        item.set_check_state(CheckState::Unchecked);
                    }
                }


                // Then, enable the mods from the profile in the UI.
                for mod_id in profile.load_order().mods() {
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

                self.mod_list_ui().model().block_signals(false);

                let game_info = self.game_selected().read().unwrap();
                let game_path = setting_path(game_info.key());
                let game_data_path = game_info.data_path(&game_path)?;

                // Then do the same for the backend. Keep in mind that if it's an autostart we have to avoid saving these changes to disk.
                if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
                    game_config.mods_mut().values_mut().for_each(|modd| { modd.set_enabled(false); });

                    for mod_id in profile.load_order().mods() {
                        if let Some(ref mut modd) = game_config.mods_mut().get_mut(mod_id) {
                            modd.set_enabled(true);
                        }
                    }

                    // Replace the current load order with the one from the profile, and update it.
                    *self.game_load_order().write().unwrap() = profile.load_order().clone();
                    let mut load_order = self.game_load_order().write().unwrap();
                    load_order.update(game_config, &game_info, &game_data_path);

                    setup_actions(&self, &game_info, &game_config, &game_path, &load_order)?;

                    // No need to do the expensive stuff on autostart, as it'll never get shown.
                    if !is_autostart {
                        load_order.save(&game_info)?;

                        let game_path = setting_path(game_info.key());
                        self.pack_list_ui().load(game_config, &game_info, &game_path, &load_order)?;
                        self.data_list_ui().set_enabled(false);
                        game_config.save(&game_info)?;
                    }
                }

                Ok(())
            }
            None => Err(anyhow!("No profile with said name found for the game selected."))
        }
    }

    pub unsafe fn save_profile(&self) -> Result<()> {
        let profile_name = self.actions_ui().profile_combobox().current_text().to_std_string();
        if profile_name.is_empty() {
            return Err(anyhow!("Profile name is empty."));
        }

        let mut profile = Profile::default();
        profile.set_id(profile_name.to_owned());
        profile.set_game(self.game_selected().read().unwrap().key().to_string());
        profile.set_load_order(self.game_load_order().read().unwrap().clone());

        self.game_profiles().write().unwrap().insert(profile_name.to_owned(), profile.clone());

        self.actions_ui().profile_model().clear();
        for profile in self.game_profiles().read().unwrap().keys() {
            self.actions_ui().profile_combobox().add_item_q_string(&QString::from_std_str(profile));
        }

        // Make sure the one we saved stays selected!!!
        self.actions_ui().profile_combobox().set_current_text(&QString::from_std_str(&profile_name));

        profile.save(&self.game_selected().read().unwrap(), &profile_name)
    }

    /// This returns the selection REVERSED!!!
    pub unsafe fn mod_list_selection(&self) -> Vec<CppBox<QModelIndex>> {
        self.mod_list_ui().mod_list_selection()
    }

    /// This returns the selection REVERSED!!!
    pub unsafe fn pack_list_selection(&self) -> Vec<CppBox<QModelIndex>> {
        self.pack_list_ui().pack_list_selection()
    }

    /// This returns the selection REVERSED!!!
    pub unsafe fn data_list_selection(&self) -> Vec<CppBox<QModelIndex>> {
        self.data_list_ui().data_list_selection()
    }

    /// This function pops up a modal asking you if you're sure you want to do an action that may result in loss of data.
    pub unsafe fn are_you_sure(&self, message: &str) -> bool {

        // Create the dialog and run it (Yes => 3, No => 4).
        QMessageBox::from_2_q_string_icon3_int_q_widget(
            &qtr("are_you_sure_title"),
            &qtr(message),
            q_message_box::Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.
            self.main_window(),
        ).exec() == 3
    }

    /// This function creates the stylesheet used for the dark theme in windows.
    pub fn dark_stylesheet() -> Result<String> {
        let mut file = File::open(ASSETS_PATH.join("dark-theme.qss"))?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        Ok(string.replace("{assets_path}", &ASSETS_PATH.to_string_lossy().replace('\\', "/")))
    }

    /// This function is used to load/reload a theme live.
    pub unsafe fn reload_theme(&self) {
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

                self.github_button().set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github.svg", ASSETS_PATH.to_string_lossy()))));
                self.actions_ui().update_icons();
            } else {
                QApplication::set_style_q_string(&QString::from_std_str("windowsvista"));
                QApplication::set_palette_1a(light_palette);
                qapp.set_style_sheet(light_style_sheet);

                self.github_button().set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github-dark.svg", ASSETS_PATH.to_string_lossy()))));
                self.actions_ui().update_icons();
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

    // String none means paste mode.
    pub unsafe fn load_order_string_dialog(&self, string: Option<String>) -> Result<Option<ImportedLoadOrderMode>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { LOAD_ORDER_STRING_VIEW_DEBUG } else { LOAD_ORDER_STRING_VIEW_RELEASE };
        let main_widget = load_template(self.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();

        let info_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "string_label")?;
        let string_text_edit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "string_text_edit")?;
        let modlist_mode_radio_button: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "modlist_mode_radio_button")?;
        let runcher_mode_radio_button: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "runcher_mode_radio_button")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        modlist_mode_radio_button.set_text(&qtr("import_string_modlist_mode"));
        runcher_mode_radio_button.set_text(&qtr("import_string_runcher_mode"));
        runcher_mode_radio_button.set_checked(true);

        let mode_group = QButtonGroup::new_1a(&dialog);

        // Configure the `Game Selected` Menu.
        mode_group.add_button_1a(&modlist_mode_radio_button);
        mode_group.add_button_1a(&runcher_mode_radio_button);

        if let Some(ref string) = string {
            dialog.set_window_title(&qtr("load_order_string_title_copy"));
            info_label.set_text(&qtr("load_order_string_info_copy"));
            string_text_edit.set_text(&QString::from_std_str(string));

            modlist_mode_radio_button.set_visible(false);
            runcher_mode_radio_button.set_visible(false);
        } else {
            dialog.set_window_title(&qtr("load_order_string_title_paste"));
            info_label.set_text(&qtr("load_order_string_info_paste"));
        }

        // If we're in "receive" mode, add a cancel button.
        if string.is_none() {
            button_box.add_button_standard_button(StandardButton::Cancel);
        }

        if dialog.exec() == 1 && string.is_none() {
            let mode = if runcher_mode_radio_button.is_checked() {
                ImportedLoadOrderMode::Runcher(string_text_edit.to_plain_text().to_std_string())
            } else {
                ImportedLoadOrderMode::Modlist(string_text_edit.to_plain_text().to_std_string())
            };

            Ok(Some(mode))
        } else {
            Ok(None)
        }
    }

    pub unsafe fn load_order_from_shareable_mod_list(&self, shareable_mod_list: &[ShareableMod]) -> Result<()> {
        if let Some(ref mut game_config) = *self.game_config().write().unwrap() {

            // Before we begin, we need to set all mods to disable. Otherwise, new load orders would get mods mixed up.
            game_config.mods_mut().iter_mut().for_each(|(_, modd)| { modd.set_enabled(false); });

            let mut missing = vec![];
            let mut wrong_hash = vec![];
            let mut ids = vec![];

            for modd in shareable_mod_list {
                match game_config.mods_mut().get_mut(modd.id()) {
                    Some(modd_local) => {
                        if let Some(path) = modd_local.paths().first() {
                            if !modd.hash().is_empty() {
                                let current_hash = try_digest(path.as_path())?;
                                if &current_hash != modd.hash() {
                                    wrong_hash.push(modd.clone());
                                }
                            }

                            modd_local.set_enabled(true);
                            ids.push(modd_local.id().to_owned());
                        }
                    },
                    None => missing.push(modd.clone()),
                }
            }

            // Once we're done updating the game config, we need to update the load order.
            //
            // We need manual order to respect the provided load order, as it may not be automatic.
            let game = self.game_selected().read().unwrap();
            let game_path = setting_path(game.key());
            let game_data_path = game.data_path(&game_path)?;

            let mut load_order = self.game_load_order().write().unwrap();
            load_order.set_mods(ids);
            load_order.set_automatic(false);
            load_order.update(game_config, &game, &game_data_path);
            load_order.save(&game)?;

            self.mod_list_ui().load(&game, game_config)?;
            self.pack_list_ui().load(game_config, &game, &game_path, &load_order)?;
            self.data_list_ui().set_enabled(false);

            setup_actions(&self, &game, &game_config, &game_path, &load_order)?;

            game_config.save(&game)?;

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
        }

        Ok(())
    }

    pub unsafe fn batch_toggle_selected_mods(&self, toggle: bool) -> Result<()> {

        // Lock the signals for the model, until the last item, so we avoid repeating full updates of the load order.
        self.mod_list_ui().model().block_signals(true);

        let selection = self.mod_list_selection();
        for selection in &selection {
            if !selection.data_1a(VALUE_IS_CATEGORY).to_bool() {
                let item = self.mod_list_ui().model().item_from_index(selection);
                if !item.is_null() && item.is_checkable() {
                    if toggle {
                        item.set_check_state(CheckState::Checked);
                    } else {
                        item.set_check_state(CheckState::Unchecked);
                    }
                }
            }
        }

        // Unlock the signals, then manually trigger a full load order rebuild.
        self.mod_list_ui().model().block_signals(false);

        if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
            for category in 0..self.mod_list_ui().model().row_count_0a() {
                let cat_item = self.mod_list_ui().model().item_2a(category, 0);
                for mod_row in 0..cat_item.row_count() {
                    let mod_item = cat_item.child_2a(mod_row, 0);
                    if !mod_item.is_null() && mod_item.is_checkable() {
                        let mod_id = mod_item.data_1a(VALUE_MOD_ID).to_string().to_std_string();
                        if let Some(ref mut modd) = game_config.mods_mut().get_mut(&mod_id) {
                            modd.set_enabled(mod_item.check_state() == CheckState::Checked);
                        }
                    }
                }
            }

            // Reload the pack view.
            let game_info = self.game_selected().read().unwrap();
            let game_path = setting_path(game_info.key());
            let game_data_path = game_info.data_path(&game_path)?;
            let mut load_order = self.game_load_order().write().unwrap();

            load_order.update(game_config, &game_info, &game_data_path);
            load_order.save(&game_info)?;

            setup_actions(&self, &game_info, &game_config, &game_path, &load_order)?;

            self.pack_list_ui().load(game_config, &game_info, &game_path, &load_order)?;
            self.data_list_ui().set_enabled(false);
            game_config.save(&game_info)?;

            Ok(())
        } else {
            Err(anyhow!("WTF?!!! game config is not writable? This is probably a bug."))
        }
    }

    pub unsafe fn create_category(&self) -> Result<()> {
        if let Some(name) = self.mod_list_ui().category_new_dialog(false)? {
            let item = QStandardItem::from_q_string(&QString::from_std_str(&name));
            item.set_data_2a(&QVariant::from_bool(true), VALUE_IS_CATEGORY);

            // New categories go second last, after the default category.
            let pos = self.mod_list_ui().model().row_count_0a() - 1;
            if pos == -1 {
                self.mod_list_ui().model().append_row_q_standard_item(item.into_ptr().as_mut_raw_ptr());
            } else {
                self.mod_list_ui().model().insert_row_int_q_standard_item(pos, item.into_ptr().as_mut_raw_ptr());
            }

            if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
                game_config.create_category(&name);

                let game = self.game_selected().read().unwrap();
                game_config.save(&game)?;
            }
        }

        Ok(())
    }

    pub unsafe fn delete_category(&self) -> Result<()> {
        let selection = self.mod_list_selection();

        if selection.iter().any(|index| index.data_1a(2).to_string().to_std_string() == DEFAULT_CATEGORY) {
            return Err(anyhow!("Dude, did you just tried to delete the {} category?!! You monster!!!", DEFAULT_CATEGORY));
        }

        for cat_to_delete in &selection {

            // Update the backend.
            if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
                if let Some(default_cat) = game_config.categories_mut().get_mut(DEFAULT_CATEGORY) {
                    let mut mods_to_reassign = (0..self.mod_list_ui().model().row_count_1a(cat_to_delete))
                        .map(|index| cat_to_delete.child(index, 0).data_1a(VALUE_MOD_ID).to_string().to_std_string())
                        .collect::<Vec<_>>();

                    default_cat.append(&mut mods_to_reassign);
                }

                // Delete the category from both, mod list and order list.
                let cat_to_delete_string = cat_to_delete.data_1a(2).to_string().to_std_string();
                game_config.delete_category(&cat_to_delete_string);
            }

            // Update the frontend.
            let mut unassigned_item = None;
            let unassigned = QString::from_std_str(DEFAULT_CATEGORY);
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

            // NOTE: We assume there is only one selection. This breaks with more.
            let cat_index = &selection[0];
            let old_cat_name = cat_index.data_1a(2).to_string().to_std_string();
            if old_cat_name == DEFAULT_CATEGORY {
                return Err(anyhow!("Dude, did you just tried to rename the {} category?!! You cannot rename perfection!!!", DEFAULT_CATEGORY));
            }

            let cat_item = self.mod_list_ui().model().item_from_index(cat_index);
            cat_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&new_cat_name)), 2);

            if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
                if let Some(cat) = game_config.categories_mut().remove(&old_cat_name) {
                    game_config.categories_mut().insert(new_cat_name.to_owned(), cat);

                    if let Some(pos) = game_config.categories_order_mut().iter().position(|x| x == &old_cat_name) {
                        game_config.categories_order_mut()[pos] = new_cat_name.to_owned();
                    }
                }
            }

            let game_info = self.game_selected().read().unwrap();
            if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
                game_config.save(&game_info)?;
            }
        }
        Ok(())
    }

    pub unsafe fn sort_category(&self) -> Result<()> {
        let selection = self.mod_list_selection();

        // NOTE: We assume there is only one selection. This breaks with more.
        let cat_index = &selection[0];
        let cat_name = cat_index.data_1a(2).to_string().to_std_string();

        // We need to sort the backend first, then remove all rows from the view, sort them like in the backend, and re-add them.
        if let Some(ref mut game_config) = *self.game_config().write().unwrap() {
            let gc_copy = game_config.clone();

            if let Some(ref mut mods) = game_config.categories_mut().get_mut(&cat_name) {
                mods.sort_by(|a, b| {
                    let mod_a = gc_copy.mods().get(a);
                    let mod_b = gc_copy.mods().get(b);
                    if let Some(mod_a) = mod_a {
                        if let Some(mod_b) = mod_b {

                            // Paths is always populated, as per the previous filter.
                            let pack_a = mod_a.paths()[0].file_name().unwrap().to_string_lossy();
                            let pack_b = mod_b.paths()[0].file_name().unwrap().to_string_lossy();

                            pack_a.cmp(&pack_b)
                        } else {
                            a.cmp(b)
                        }
                    } else {
                        a.cmp(b)
                    }
                });

                let mut rows = vec![];
                let cat_item = self.mod_list_ui().model().item_from_index(cat_index);
                for index in (0..self.mod_list_ui().model().row_count_1a(cat_index)).rev() {
                    rows.push(cat_item.take_row(index).into_ptr());
                }

                for mod_id in &**mods {
                    if let Some(pos) = rows.iter().position(|row| &row.value_1a(0).data_1a(VALUE_MOD_ID).to_string().to_std_string() == mod_id) {
                        let row = rows.remove(pos);
                        cat_item.append_row_q_list_of_q_standard_item(row.as_ref().unwrap());
                    }
                }

                let game_info = self.game_selected().read().unwrap();
                game_config.save(&game_info)?;
            }
        }

        Ok(())
    }

    /// Parent is model means dest_parent is a modelindex FROM THE MODEL, NOT FROM THE VIEW.
    pub unsafe fn move_category(&self, dest_parent: Ref<QModelIndex>, dest_row: i32, parent_is_model: bool) -> Result<()> {

        // Rare case, but possible due to selection weirdness.
        let selection = self.mod_list_selection();
        if selection.is_empty() {
            return Ok(());
        }

        // Limitation: we can only move together categories or mods, not both.
        let mut cats = false;
        let mut mods = false;
        selection.iter()
            .for_each(|selection| {
                if selection.data_1a(VALUE_IS_CATEGORY).to_bool() {
                    cats = true;
                } else {
                    mods = true;
                }
            });

        if mods == cats {
            return Err(anyhow!("You cannot move categories and mods at the same time."));
        }

        if cats && selection.iter().any(|selection| selection.data_0a().to_string().to_std_string() == DEFAULT_CATEGORY) {
            return Err(anyhow!("Cannot move the default category {}.", DEFAULT_CATEGORY));
        }

        // dest_parent may be invalid if we're dropping between categories.
        if cats && dest_parent.is_valid() {
            return Ok(());
        }

        // dest_parent may be valid for mods if we're dropping inside a category. If we drop in a category item, the parent is invalid.
        if mods && dest_parent.is_valid() && !dest_parent.data_1a(VALUE_IS_CATEGORY).to_bool() {
            return Ok(());
        }

        if let Some(ref mut game_config) = *self.game_config().write().unwrap() {

            // Categories move.
            //
            // The offset is so we get the correct destination after we remove the categories that may be before the destination.
            if cats {
                let cats_to_move = selection.iter().rev().map(|x| x.data_0a().to_string().to_std_string()).collect::<Vec<_>>();
                let offset = cats_to_move.iter()
                    .filter_map(|cat| game_config.categories_order().iter().position(|cat2| cat == cat2))
                    .filter(|pos| pos < &(dest_row as usize))
                    .count();

                game_config.categories_order_mut().retain(|x| !cats_to_move.contains(x));

                for (index, cat) in cats_to_move.iter().enumerate() {
                    let pos = dest_row as usize + index - offset;
                    game_config.categories_order_mut().insert(pos, cat.to_owned());
                }

                // Visual move.
                let mut rows = selection.iter().map(|x| self.mod_list_ui().model().take_row(x.row())).collect::<Vec<_>>();
                rows.reverse();

                for (index, row) in rows.iter().enumerate() {
                    let pos = dest_row as usize + index - offset;
                    self.mod_list_ui().model().insert_row_int_q_list_of_q_standard_item(pos as i32, row);
                }
            }

            // Mods move.
            else if mods {
                let mods_to_move = selection.iter().rev().map(|x| x.data_1a(VALUE_MOD_ID).to_string().to_std_string()).collect::<Vec<_>>();

                // Invalid means we're dropping in a category item.
                let category_index = self.mod_list_ui().filter().index_2a(dest_row, 0);
                let category_index_visual = if dest_parent.is_valid() {
                    dest_parent
                } else {
                    category_index.as_ref()
                };

                // If we have no parent (dropping in the category) add at the start.
                let dest_row_final = if dest_parent.is_valid() {
                    dest_row
                } else {
                    0
                };

                let category_index_logical = if parent_is_model {
                    self.mod_list_ui().model().index_2a(category_index_visual.row(), category_index_visual.column())
                } else {
                    self.mod_list_ui().filter().map_to_source(category_index_visual)
                };

                let dest_category = category_index_logical.data_0a().to_string().to_std_string();
                let mut offset = 0;
                if let Some(dest_mods) = game_config.categories().get(&dest_category) {
                    offset = mods_to_move.iter()
                        .filter(|mod_id| dest_mods.contains(mod_id))
                        .filter_map(|mod_id| dest_mods.iter().position(|mod_id2| mod_id == mod_id2))
                        .filter(|pos| pos < &(dest_row_final as usize))
                        .count();
                }

                for mods in game_config.categories_mut().values_mut() {
                    mods.retain(|x| !mods_to_move.contains(x));
                }

                if let Some(dest_mods) = game_config.categories_mut().get_mut(&dest_category) {
                    for (index, mod_id) in mods_to_move.iter().enumerate() {
                        let pos: i32 = dest_row_final + index as i32 - offset as i32;
                        dest_mods.insert(pos as usize, mod_id.to_owned());
                    }
                }

                // Visual move.
                let mut rows = selection.iter().map(|x| self.mod_list_ui().model().item_from_index(&x.parent()).take_row(x.row())).collect::<Vec<_>>();
                rows.reverse();

                let dest_item = self.mod_list_ui().model().item_from_index(&category_index_logical);
                for (index, row) in rows.iter().enumerate() {
                    let pos: i32 = dest_row_final + index as i32 - offset as i32;
                    if pos == dest_item.row_count() {
                        dest_item.append_row_q_list_of_q_standard_item(row);
                    } else {
                        dest_item.insert_row_int_q_list_of_q_standard_item(pos, row);
                    }
                }
            }

            let game_info = self.game_selected().read().unwrap();
            game_config.save(&game_info)?;
        }

        Ok(())
    }

    pub unsafe fn generate_move_to_category_submenu(app_ui: &Rc<AppUI>) {
        if let Some(ref game_config) = *app_ui.game_config().read().unwrap() {
            app_ui.mod_list_ui().categories_send_to_menu().clear();

            for category in game_config.categories_order() {
                let action = app_ui.mod_list_ui().categories_send_to_menu().add_action_q_string(&QString::from_std_str(category));
                let slot = SlotNoArgs::new(app_ui.mod_list_ui().categories_send_to_menu(), clone!(
                    category,
                    app_ui => move || {
                        if let Some(item) = app_ui.mod_list_ui().category_item(&category) {
                            let index = item.index();
                            if let Err(error) = app_ui.move_category(index.as_ref(), item.row_count(), true) {
                                show_dialog(app_ui.main_window(), error, false);
                            }
                        }
                    }
                ));

                action.triggered().connect(&slot);
            }
        }
    }

    pub unsafe fn move_pack(&self, new_position: i32) -> Result<()> {

        // Rare case, but possible due to selection weirdness.
        let selection = self.pack_list_selection();
        if selection.is_empty() {
            return Ok(());
        }

        // Do NOT allow moving movie packs.
        if selection.iter().any(|x| self.pack_list_ui().model().index_2a(x.row(), 1).data_0a().to_string().to_std_string() != PFHFileType::Mod.to_string()) {
            return Ok(());
        }

        // Do NOT allow placing a mod pack under a movie pack.
        let mut load_order = self.game_load_order().write().unwrap();
        if !load_order.movies().is_empty() && new_position as usize > load_order.mods().len() {
            return Ok(());
        }

        // This one is easier than with categories: we just calculate the offset, take the items at selected positions, then re-add them in their new position.
        let packs_to_move = selection.iter().rev().map(|x| x.data_1a(VALUE_MOD_ID).to_string().to_std_string()).collect::<Vec<_>>();
        let offset = load_order.mods().iter()
            .enumerate()
            .filter(|(index, mod_id)| (index < &(new_position as usize) && packs_to_move.contains(mod_id)))
            .count();

        load_order.mods_mut().retain(|mod_id| !packs_to_move.contains(mod_id));
        for (index, mod_id) in packs_to_move.iter().enumerate() {
            let pos: i32 = new_position + index as i32 - offset as i32;
            load_order.mods_mut().insert(pos as usize, mod_id.to_owned());
        }
        let game_info = self.game_selected().read().unwrap();
        load_order.save(&game_info)?;

        // Visual move.
        let mut rows = selection.iter().map(|x| self.pack_list_ui().model().take_row(x.row()).into_ptr()).collect::<Vec<_>>();
        rows.reverse();

        for (index, row) in rows.iter().enumerate() {
            let pos = new_position as usize + index - offset;
            self.pack_list_ui().model().insert_row_int_q_list_of_q_standard_item(pos as i32, row.as_ref().unwrap());
        }

        for row in 0..self.pack_list_ui().model().row_count_0a() {
            let item = self.pack_list_ui().model().item_2a(row, 3);
            if !item.is_null() {
                item.set_data_2a(&QVariant::from_int(row), 2);
            }
        }

        Ok(())
    }

    pub unsafe fn generate_open_in_tools_submenu(app_ui: &Rc<AppUI>) {
        let menu = app_ui.mod_list_ui().open_in_tool_menu();
        menu.clear();

        let mut tools = app_ui.tools().read().unwrap().clone();
        tools.tools_mut().sort_by(|tool_a, tool_b| tool_a.name().cmp(tool_b.name()));

        let game = app_ui.game_selected().read().unwrap();
        for tool in tools.tools() {
            if tool.games().iter().any(|x| x == game.key()) {
                let action = menu.add_action_q_string(&QString::from_std_str(tool.name()));
                let slot = SlotNoArgs::new(menu, clone!(
                    tool,
                    app_ui => move || {
                        if let Some(ref game_config) = *app_ui.game_config().read().unwrap() {
                            let selection = app_ui.mod_list_selection();
                            let mod_index = &selection[0];
                            let mod_id = mod_index.data_1a(VALUE_MOD_ID).to_string().to_std_string();

                            if let Some(modd) = game_config.mods().get(&mod_id) {
                                if let Some(path) = modd.paths().first() {
                                    if let Err(error) = std::process::Command::new(tool.path().to_string_lossy().to_string())
                                        .arg(path.to_string_lossy().to_string())
                                        .spawn() {
                                        show_dialog(app_ui.main_window(), error, false);
                                    }
                                }
                            }
                        }
                    }
                ));

                action.triggered().connect(&slot);
            }
        }
    }

    /// Function to move files from /content to /secondary, or /data.
    fn move_to_destination(&self, data_path: &Path, secondary_path: &Option<PathBuf>, steam_user_id: &str, game: &GameInfo, modd: &mut Mod, mod_name: &str, pack: &mut Pack, new_pack_type: bool) -> Result<()> {

        // Sometimes they come canonicalized, sometimes dont. This kinda fixes it.
        let new_path_in_data = data_path.join(mod_name);
        let new_path_in_data = std::fs::canonicalize(new_path_in_data.clone()).unwrap_or(new_path_in_data);
        let mut in_secondary = false;

        // First try to move it to secondary if it's not in /data. Only if it's not in /data already.
        if let Some(ref secondary_path) = &secondary_path {
            if !new_path_in_data.is_file() {
                let new_path_in_secondary = secondary_path.join(mod_name);

                // Copy the files unless it exists and its ours.
                if (!new_path_in_secondary.is_file() || (new_path_in_secondary.is_file() && steam_user_id != modd.creator())) && pack.save(Some(&new_path_in_secondary), game, &None).is_ok() {
                    if !modd.paths().contains(&new_path_in_secondary) {
                        modd.paths_mut().insert(0, new_path_in_secondary);
                    }

                    if new_pack_type {
                        modd.set_pack_type(pack.pfh_file_type());
                    }

                    in_secondary = true;
                }
            }
        }

        // If the move to secondary failed, try to do the same with /data.
        if !in_secondary {

            // Copy the files unless it exists and its ours.
            if (!new_path_in_data.is_file() || (new_path_in_data.is_file() && steam_user_id != modd.creator())) && pack.save(Some(&new_path_in_data), game, &None).is_ok() {
                if !modd.paths().contains(&new_path_in_data) {
                    modd.paths_mut().insert(0, new_path_in_data);
                }

                if new_pack_type {
                    modd.set_pack_type(pack.pfh_file_type());
                }
            }
        }

        Ok(())
    }

    /// Function to generate a pack from a Shogun 2 map bin data.
    fn generate_map_pack(&self, game: &GameInfo, data_dec: &[u8], pack_name: &str, map_name: &str) -> Result<Pack> {

        // Get all the files into memory to generate its pack.
        let mut files = HashMap::new();
        let mut data_dec = Cursor::new(data_dec);
        loop {

            // At the end of the last file there's a 0A 00 00 00 that doesn't seem to be part of a file.
            let len = data_dec.len()?;
            if len < 4 || data_dec.position() >= len - 4 {
                break;
            }

            let file_name = data_dec.read_string_u16_0terminated()?;
            let size = data_dec.read_u64()?;
            let data = data_dec.read_slice(size as usize, false)?;

            files.insert(file_name, data);
        }

        let mut pack = Pack::new_with_name_and_version(pack_name, game.pfh_version_by_file_type(PFHFileType::Mod));
        let spec_path = format!("battleterrain/presets/{}/", &map_name);

        // We need to add the files under /BattleTerrain/presets/map_name
        for (file_name, file_data) in &files {
            let rfile_path = spec_path.to_owned() + file_name;
            let mut rfile = RFile::new_from_vec(file_data, FileType::Unknown, 0, &rfile_path);
            let _ = rfile.guess_file_type();
            let _ = pack.insert(rfile);
        }

        // We also need to generate a battles table for our mod, so it shows up ingame, and a loc table, so it has a name ingame.
        //
        // The data for all of this needs to be parsed from the map_info.xml file.
        if let Some(map_info) = files.get("map_info.xml") {
            if let Ok(map_info) = String::from_utf8(map_info.to_vec()) {
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    let table_name = "battles_tables";
                    let table_version = 4;
                    if let Some(definition) = schema.definition_by_name_and_version(table_name, table_version) {

                        // DB
                        let patches = schema.patches_for_table(table_name);
                        let mut file = DB::new(definition, patches, table_name);
                        let mut row = file.new_row();

                        if let Some(column) = file.column_position_by_name("key") {
                            if let Some(DecodedData::StringU16(key)) = row.get_mut(column) {
                                *key = map_name.to_string();
                            }
                        }

                        if let Some(column) = file.column_position_by_name("type") {
                            if let Some(DecodedData::StringU16(battle_type)) = row.get_mut(column) {
                                if let Some(battle_type_xml) = REGEX_MAP_INFO_TYPE.captures(&map_info) {
                                    if let Some(battle_type_xml) = battle_type_xml.get(1) {
                                        if battle_type_xml.as_str() == "land" {
                                            *battle_type = "classic".to_string();
                                        } else {
                                            *battle_type = battle_type_xml.as_str().to_string();
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(column) = file.column_position_by_name("specification") {
                            if let Some(DecodedData::StringU16(specification_path)) = row.get_mut(column) {
                                *specification_path = spec_path.to_owned();
                            }
                        }

                        if let Some(column) = file.column_position_by_name("screenshot_path") {
                            if let Some(DecodedData::OptionalStringU16(screenshot_path)) = row.get_mut(column) {
                                *screenshot_path = spec_path + "icon.tga";
                            }
                        }

                        if let Some(column) = file.column_position_by_name("team_size_1") {
                            if let Some(DecodedData::I32(team_size_1)) = row.get_mut(column) {
                                if let Some(team_size_1_xml) = REGEX_MAP_INFO_TEAM_SIZE_1.captures(&map_info) {
                                    if let Some(team_size_1_xml) = team_size_1_xml.get(1) {
                                        if let Ok(team_size_1_xml) = team_size_1_xml.as_str().parse::<i32>() {
                                            *team_size_1 = team_size_1_xml;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(column) = file.column_position_by_name("team_size_2") {
                            if let Some(DecodedData::I32(team_size_2)) = row.get_mut(column) {
                                if let Some(team_size_2_xml) = REGEX_MAP_INFO_TEAM_SIZE_2.captures(&map_info) {
                                    if let Some(team_size_2_xml) = team_size_2_xml.get(1) {
                                        if let Ok(team_size_2_xml) = team_size_2_xml.as_str().parse::<i32>() {
                                            *team_size_2 = team_size_2_xml;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(column) = file.column_position_by_name("release") {
                            if let Some(DecodedData::Boolean(value)) = row.get_mut(column) {
                                *value = true;
                            }
                        }

                        if let Some(column) = file.column_position_by_name("multiplayer") {
                            if let Some(DecodedData::Boolean(value)) = row.get_mut(column) {
                                *value = true;
                            }
                        }

                        if let Some(column) = file.column_position_by_name("singleplayer") {
                            if let Some(DecodedData::Boolean(value)) = row.get_mut(column) {
                                *value = true;
                            }
                        }

                        if let Some(column) = file.column_position_by_name("defender_funds_ratio") {
                            if let Some(DecodedData::F32(funds_ratio)) = row.get_mut(column) {
                                if let Some(funds_ratio_xml) = REGEX_MAP_INFO_DEFENDER_FUNDS_RATIO.captures(&map_info) {
                                    if let Some(funds_ratio_xml) = funds_ratio_xml.get(1) {
                                        if let Ok(funds_ratio_xml) = funds_ratio_xml.as_str().parse::<f32>() {
                                            *funds_ratio = funds_ratio_xml;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(column) = file.column_position_by_name("has_key_buildings") {
                            if let Some(DecodedData::Boolean(value)) = row.get_mut(column) {
                                if let Some(has_key_buildings_xml) = REGEX_MAP_INFO_HAS_KEY_BUILDINGS.captures(&map_info) {
                                    if let Some(has_key_buildings_xml) = has_key_buildings_xml.get(1) {
                                        if let Ok(has_key_buildings_xml) = has_key_buildings_xml.as_str().parse::<bool>() {
                                            *value = has_key_buildings_xml;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(column) = file.column_position_by_name("matchmaking") {
                            if let Some(DecodedData::Boolean(value)) = row.get_mut(column) {
                                *value = true;
                            }
                        }

                        file.data_mut().push(row);
                        let rfile_decoded = RFileDecoded::DB(file);
                        let rfile_path = format!("db/battles_tables/{}", map_name);
                        let rfile = RFile::new_from_decoded(&rfile_decoded, 0, &rfile_path);
                        let _ = pack.insert(rfile);

                        // Loc
                        let mut file = Loc::new();

                        if let Some(display_name) = REGEX_MAP_INFO_DISPLAY_NAME.captures(&map_info) {
                            if let Some(display_name) = display_name.get(1) {
                                let mut row = file.new_row();

                                row[0] = DecodedData::StringU16(format!("battles_localised_name_{}", map_name));
                                row[1] = DecodedData::StringU16(display_name.as_str().to_string());

                                file.data_mut().push(row);
                            }
                        }

                        if let Some(description) = REGEX_MAP_INFO_DESCRIPTION.captures(&map_info) {
                            if let Some(description) = description.get(1) {
                                let mut row = file.new_row();

                                row[0] = DecodedData::StringU16(format!("battles_description_{}", map_name));
                                row[1] = DecodedData::StringU16(description.as_str().to_string());

                                file.data_mut().push(row);
                            }
                        }

                        let rfile_decoded = RFileDecoded::Loc(file);
                        let rfile_path = format!("text/db/{}.loc", map_name);
                        let rfile = RFile::new_from_decoded(&rfile_decoded, 0, &rfile_path);
                        let _ = pack.insert(rfile);
                    }
                }
            }
        }

        Ok(pack)
    }

    pub unsafe fn update_mod_list_with_online_data(&self, receiver: &Option<Receiver<Response>>) -> Result<()> {
        if let Some(receiver) = receiver {
            let response = CENTRAL_COMMAND.recv_try(receiver);
            match response {
                Response::VecMod(workshop_items) => {
                    let mut game_config = self.game_config().write().unwrap();
                    if let Some(ref mut game_config) = *game_config {
                        let game = self.game_selected().read().unwrap().clone();
                        let game_path = setting_path(game.key());

                        if populate_mods_with_online_data(game_config.mods_mut(), &workshop_items).is_ok() {

                            // Shogun 2 uses two types of mods:
                            // - Pack mods turned binary: they're pack mods with a few extra bytes at the beginning. RPFM lib is capable to open them, save them as Packs, then do one of these:
                            //   - If the mod pack is in /data, we copy it there.
                            //   - If the mod pack is not /data and we have a secondary folder configured, we copy it there.
                            //   - If the mod pack is not /data and we don't have a secondary folder configured, we copy it to /data.
                            // - Map mods. These are zlib-compressed lists of files. Their encoding turned to be quite simple:
                            //   - Null-terminated StringU16: File name.
                            //   - u64: File data size.
                            //   - [u8; size]: File data.
                            //   - Then at the end there is an u32 with a 0A that we ignore.
                            //
                            // Other games may also use the first type, but most modern uploads are normal Packs.
                            //
                            // So, once population is done, we need to do some post-processing. Our mods need to be moved to either /data or /secondary if we don't have them there.
                            // Shogun 2 mods need to be turned into packs and moved to either /data or /secondary.
                            let steam_user_id = crate::mod_manager::integrations::store_user_id(&game)?.to_string();
                            let secondary_path = secondary_mods_path(game.key()).ok();
                            let game_data_path = game.data_path(&game_path);

                            for modd in game_config.mods_mut().values_mut() {
                                if let Some(last_path) = modd.paths().last() {

                                    // Only copy bins which are not yet in the destination folder and which are not made by the steam user.
                                    let legacy_mod = modd.id().ends_with(".bin") && !modd.file_name().is_empty();
                                    if legacy_mod && modd.file_name().ends_with(".pack"){

                                        // This is for Packs. Map mods use a different process.
                                        if let Ok(mut pack) = Pack::read_and_merge(&[last_path.to_path_buf()], &game, true, false, false) {
                                            if let Ok(ref data_path) = game_data_path {

                                                let mod_name = if legacy_mod {
                                                    if let Some(name) = modd.file_name().split('/').last() {
                                                        name.to_string()
                                                    } else {
                                                        modd.id().to_string()
                                                    }
                                                } else {
                                                    modd.id().to_string()
                                                };

                                                let _ = self.move_to_destination(data_path, &secondary_path, &steam_user_id, &game, modd, &mod_name, &mut pack, false);
                                            }
                                        }
                                    }

                                    // If it's not a pack, but is reported as a legacy mod, is a map mod from Shogun 2.
                                    else if legacy_mod && game.key() == KEY_SHOGUN_2 {
                                        if let Some(name) = modd.file_name().clone().split('/').last() {

                                            // Maps only contain a folder name. We need to change it into a pack name.
                                            let name = name.replace(" ", "_");
                                            let pack_name = name.to_owned() + ".pack";

                                            if let Ok(ref data_path) = game_data_path {
                                                if let Ok(file) = File::open(last_path) {
                                                    let mut file = BufReader::new(file);
                                                    if let Ok(metadata) = file.get_ref().metadata() {
                                                        let mut data = Vec::with_capacity(metadata.len() as usize);
                                                        if file.read_to_end(&mut data).is_ok() {

                                                            let reader = BufReader::new(Cursor::new(data.to_vec()));
                                                            let mut decompressor = ZlibDecoder::new(reader);
                                                            let mut data_dec = vec![];

                                                            if decompressor.read_to_end(&mut data_dec).is_ok() {
                                                                let mut pack = self.generate_map_pack(&game, &data_dec, &pack_name, &name)?;

                                                                // Once done generating the pack, just do the same as with normal mods.
                                                                let _ = self.move_to_destination(data_path, &secondary_path, &steam_user_id, &game, modd, &pack_name, &mut pack, false);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Before continuing, we need to do some cleaning. There's a chance that due to the order of operations done to populate the mod list
                            // Some legacy packs get split into two distinct mods. We need to detect them and clean them up here.
                            let alt_names = game_config.mods()
                                .par_iter()
                                .filter_map(|(_, modd)| modd.alt_name())
                                .collect::<Vec<_>>();

                            for alt_name in &alt_names {
                                game_config.mods_mut().remove(alt_name);
                                game_config.categories_mut().iter_mut().for_each(|(_, mods)| {
                                    mods.retain(|modd| modd != alt_name);
                                });
                            }

                            game_config.save(&game)?;

                            // If we got a successfull network update, then proceed to update the UI with the new data.
                            // It's faster than a full rebuild, and looks more modern and async.
                            self.mod_list_ui().update(&game, game_config.mods(), &alt_names)?;

                            // Reload the pack list, as it may have changed in some cases (Shogun 2).
                            let load_order = self.game_load_order().read().unwrap();
                            self.pack_list_ui().load(game_config, &game, &game_path, &load_order)?;
                        }
                    }
                }

                // Ignore errors on network requests for now.
                Response::Error(_) => {},
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),

            }
        }

        Ok(())
    }

    pub unsafe fn upload_mod_to_workshop(&self) -> Result<Option<()>> {
        let selection = self.mod_list_selection();
        if selection.len() == 1 && !selection[0].data_1a(VALUE_IS_CATEGORY).to_bool() {
            let mod_id = selection[0].data_1a(VALUE_MOD_ID).to_string().to_std_string();
            let game_config = self.game_config().read().unwrap();
            if let Some(ref game_config) = *game_config {
                if let Some(modd) = game_config.mods().get(&mod_id) {
                    let game = self.game_selected().read().unwrap();

                    // Before loading the dialog, we need to do some sanity checks, which include:
                    // - Check if the mod was previously uploaded.
                    // - Retrieve updated data from the workshop if the file is already uploaded.
                    //
                    // We use the updated data to populate the dialog. If it was never uploaded (no steam id), we just load the dialog.
                    let mod_data = if let Some(steam_id) = modd.steam_id() {
                        request_pre_upload_info(&game, steam_id)?
                    } else {
                        PreUploadInfo::default()
                    };

                    // If no errors were found, load the UI Template.
                    let template_path = if cfg!(debug_assertions) { WORKSHOP_UPLOAD_VIEW_DEBUG } else { WORKSHOP_UPLOAD_VIEW_RELEASE };
                    let main_widget = load_template(self.main_window(), template_path)?;
                    let dialog = main_widget.static_downcast::<QDialog>();

                    let title_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "title_label")?;
                    let description_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "description_label")?;
                    let changelog_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "changelog_label")?;
                    let tag_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "tag_label")?;
                    let visibility_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "visibility_label")?;

                    let title_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "title_line_edit")?;
                    let description_text_edit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "description_text_edit")?;
                    let changelog_text_edit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "changelog_text_edit")?;
                    let tag_combo_box: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "tag_combo_box")?;
                    let visibility_combo_box: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "visibility_combo_box")?;

                    let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
                    button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

                    dialog.set_window_title(&qtr("upload_to_workshop_title"));
                    title_label.set_text(&qtr("upload_workshop_title"));
                    description_label.set_text(&qtr("upload_workshop_description"));
                    changelog_label.set_text(&qtr("upload_workshop_changelog"));
                    tag_label.set_text(&qtr("upload_workshop_tag"));
                    visibility_label.set_text(&qtr("upload_workshop_visibility"));

                    let tags = game.steam_workshop_tags()?;
                    for tag in &tags {
                        tag_combo_box.add_item_q_string(&QString::from_std_str(tag));
                    }

                    visibility_combo_box.add_item_q_string(&qtr("upload_workshop_visibility_public"));
                    visibility_combo_box.add_item_q_string(&qtr("upload_workshop_visibility_friends_only"));
                    visibility_combo_box.add_item_q_string(&qtr("upload_workshop_visibility_private"));
                    visibility_combo_box.add_item_q_string(&qtr("upload_workshop_visibility_unlisted"));

                    // If we got data from the workshop, populate it with that.
                    if mod_data.published_file_id > 0 {
                        title_line_edit.set_text(&QString::from_std_str(mod_data.title));
                        description_text_edit.set_plain_text(&QString::from_std_str(mod_data.description));
                        changelog_text_edit.set_plain_text(&QString::from_std_str("me forgot changelog. Me sorry."));

                        // For tag selection, we expect to have two. We need to pick the one that's not "mod".
                        if let Some(selected_tag) = mod_data.tags.iter().find_or_first(|x| &**x != "mod") {
                            tag_combo_box.set_current_text(&QString::from_std_str(selected_tag));
                        }

                        visibility_combo_box.set_current_index(match mod_data.visibility {
                            PublishedFileVisibilityDerive::Public => 0,
                            PublishedFileVisibilityDerive::FriendsOnly => 1,
                            PublishedFileVisibilityDerive::Private => 2,
                            PublishedFileVisibilityDerive::Unlisted => 3,
                        });
                    }

                    // Otherwise, put default data there.
                    else {
                        title_line_edit.set_text(&QString::from_std_str(modd.id()));
                        changelog_text_edit.set_plain_text(&QString::from_std_str("Initial release."));
                        visibility_combo_box.set_current_index(2);
                    }

                    if dialog.exec() == 1 {
                        let mut title = title_line_edit.text().to_std_string();
                        let description = description_text_edit.to_plain_text().to_std_string();
                        let changelog = changelog_text_edit.to_plain_text().to_std_string();
                        let tags = vec![tag_combo_box.current_text().to_std_string()];
                        let visibility = visibility_combo_box.current_index() as u32;

                        // We need at least a title. So if we don't have one, use the default one.
                        if title.is_empty() {
                            title = modd.id().to_string();
                        }

                        crate::mod_manager::integrations::upload_mod_to_workshop(&game, modd, &title, &description, &tags, &changelog, &Some(visibility), true).map(Some)
                    } else {
                        Ok(None)
                    }

                    // All the following elses should never really trigger unless it's a bug.
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub unsafe fn download_subscribed_mods(&self, published_file_ids: &Option<Vec<String>>) -> Result<()> {
        self.toggle_main_window(false);

        crate::mod_manager::integrations::download_subscribed_mods(&self.game_selected().read().unwrap(), published_file_ids)?;

        self.toggle_main_window(true);

        // Once done, do a reload of the mod list.
        self.actions_ui().reload_button().click();

        Ok(())
    }

    pub unsafe fn check_logs(&self, game: &GameInfo, game_path: &Path, start_date: &SystemTime) -> Result<()> {

        // NOTE: THIS IS A HACK. WE NEED TO USE SOME KIND OF CACHED DATA, NOT REMAKE IT HERE!!!!
        let game_config = self.game_config().read().unwrap().clone().unwrap();
        let load_order = self.game_load_order().read().unwrap();
        let pack = self.data_list_ui().generate_data(&game_config, game, game_path, &load_order)?;

        let vanilla_paths = game.ca_packs_paths(game_path)?;
        let files = files_from_subdir(game_path, false)?;
        let paths = files.iter()
            .filter(|path| {
                let modified = path.metadata().unwrap().modified().unwrap();
                //let start_date = &SystemTime::from(std::time::UNIX_EPOCH);
                modified > *start_date && path.extension().is_some() && path.extension().unwrap() == "txt"
            })
            .collect::<Vec<_>>();

        let mut breaks = vec![];
        for path in &paths {
            let mut data = String::new();
            let mut file = BufReader::new(File::open(path)?);

            // This fails in the clockwork one due to being windows-1252
            if file.read_to_string(&mut data).is_ok() {

                // Normal error.
                /*
                ********************
                SCRIPT ERROR, timestamp <375.0s>
                ERROR - SCRIPT HAS FAILED - event callback was called after receiving event [WorldStartRound] but the script failed with this error message:
                [string "script\campaign\dynamic_disasters\disaster_the_great_bastion_improved.lua"]:609: attempt to get length of field '?' (a nil value)

                The callstack of the failed script is:

                stack traceback:
                    [string "script\campaign\dynamic_disasters\disaster_the_great_bastion_improved.lua"]:609: in function 'trigger_pre_invasion_1'
                    [string "script\campaign\dynamic_disasters\disaster_the_great_bastion_improved.lua"]:313: in function 'callback'
                    [string "script\_lib\lib_core.lua"]:1930: in function <[string "script\_lib\lib_core.lua"]:1930>
                    [C]: in function 'xpcall'
                    [string "script\_lib\lib_core.lua"]:1930: in function 'event_protected_callback'
                    [string "script\_lib\lib_core.lua"]:1991: in function 'event_callback'
                    [string "script\_lib\lib_core.lua"]:2051: in function <[string "script\_lib\lib_core.lua"]:2051>

                The callstack of the script which established the failed listener is:
                stack traceback:
                    [string "script\_lib\lib_core.lua"]:1908: in function 'add_listener'
                    [string "script\campaign\dynamic_disasters\disaster_the_great_bastion_improved.lua"]:260: in function 'set_status'
                    [string "script\campaign\dynamic_disasters\disaster_the_great_bastion_improved.lua"]:565: in function 'trigger_the_great_bastion_improved'
                    [string "script\campaign\dynamic_disasters\disaster_the_great_bastion_improved.lua"]:486: in function 'start'
                    [string "script\campaign\mod\dynamic_disasters.lua"]:606: in function <[string "script\campaign\mod\dynamic_disasters.lua"]:536>
                    (tail call): ?
                    [string "script\_lib\lib_core.lua"]:1930: in function <[string "script\_lib\lib_core.lua"]:1930>
                    [C]: in function 'xpcall'
                    [string "script\_lib\lib_core.lua"]:1930: in function 'event_protected_callback'
                    [string "script\_lib\lib_core.lua"]:1991: in function 'event_callback'
                    [string "script\_lib\lib_core.lua"]:2051: in function <[string "script\_lib\lib_core.lua"]:2051>
                ********************
                 */
                let normal_errors = data.match_indices("SCRIPT ERROR, timestamp").collect::<Vec<_>>();
                for (start_error, _) in normal_errors {
                    if let Some(end_error) = data[start_error..].find("********************") {
                        let message = data[start_error..start_error + end_error].to_owned();
                        let mut script_break = ScriptBreak {
                            full_log: message.to_owned(),
                            ..Default::default()
                        };

                        let start_path = "[string \"";
                        let end_path = "\"]:";
                        let mut paths = vec![];
                        for (start_path_pos, _) in message.match_indices(start_path) {
                            if let Some(end_path_pos) = message[start_path_pos + 9..].find(end_path) {
                                let path = message[start_path_pos + 9..start_path_pos + 9 + end_path_pos].replace("\\", "/");
                                paths.push(path);
                            }
                        }

                        // NOTE: pack finding only works if the pack that caused it is in the current run. Take that into account for tests.
                        for path in &paths {
                            if let Some(file) = pack.file(path, true) {
                                if let Some(pack_name) = file.container_name() {
                                    if !pack_name.is_empty() && vanilla_paths.iter().all(|x| &x.file_name().unwrap().to_string_lossy().to_string() != pack_name) {
                                        script_break.posible_pack = pack_name.to_owned();

                                        // This is only valid in newer games!!!
                                        let modd = game_config.mods().get(pack_name);
                                        script_break.posible_pack_mod = modd
                                            .map(|modd| modd.name().to_string())
                                            .unwrap_or_default();
                                        script_break.posible_pack_link = modd.and_then(|modd| modd.steam_id()
                                            .as_ref()
                                            .map(|id| format!("https://steamcommunity.com/sharedfiles/filedetails/?id={}", id))
                                        );
                                        break;
                                    }
                                }
                            }
                        }

                        breaks.push(script_break);
                    }
                }

                // Big Fat error.
                /*
                [out] <1593.9s>  BIG FAT SCRIPT ERROR
                [out] <1593.9s>  [string "script\campaign\mod\meh_blightwing_duchy_campaign_features.lua"]:63: attempt to call method 'character_subtype_key' (a nil value)
                [out] <1593.9s>  stack traceback:
                    [string "script\_lib\mod\pj_error_wrapping.lua"]:50: in function 'condition'
                    [string "script\_lib\lib_core.lua"]:1928: in function <[string "script\_lib\lib_core.lua"]:1928>
                    [C]: in function 'xpcall'
                    [string "script\_lib\lib_core.lua"]:1928: in function 'event_protected_callback'
                    [string "script\_lib\lib_core.lua"]:1965: in function 'event_callback'
                    [string "script\_lib\lib_core.lua"]:2051: in function <[string "script\_lib\lib_core.lua"]:2051>
                [out] <1594.1s>   & Removing effect bundle [wh3_main_bundle_force_crackdown_corruption] from military force with cqi [80]
                [out] <1594.1s>   & Removing effect bundle [ovn_fimir_fog_diktat_empty] from the force of character with cqi [159]
                [out] <1594.1s>  DrunkFlamingo: Checking faction ally outposts for faction: wh2_dlc17_bst_malagor (temp tomb king ally fix)

                 */
                let big_fat_errors = data.match_indices("BIG FAT SCRIPT ERROR").collect::<Vec<_>>();
                for (start_error, _) in big_fat_errors {

                    // For end we use the third out.
                    if let Some(first) = data[start_error..].find("[out]") {
                        if let Some(second) = data[start_error + first + 3 ..].find("[out]") {
                            if let Some(end_error) = data[start_error + first + 3 + second + 3..].find("[out]") {
                                let message = data[start_error..start_error + first + 3 + second + 3 + end_error].to_owned();
                                let mut script_break = ScriptBreak {
                                    full_log: message.to_owned(),
                                    ..Default::default()
                                };

                                let start_path = "[string \"";
                                let end_path = "\"]:";
                                let mut paths = vec![];
                                for (start_path_pos, _) in message.match_indices(start_path) {
                                    if let Some(end_path_pos) = message[start_path_pos + 9..].find(end_path) {
                                        let path = message[start_path_pos + 9..start_path_pos + 9 + end_path_pos].replace("\\", "/");
                                        paths.push(path);
                                    }
                                }

                                // NOTE: pack finding only works if the pack that caused it is in the current run. Take that into account for tests.
                                for path in &paths {
                                    if let Some(file) = pack.file(path, true) {
                                        if let Some(pack_name) = file.container_name() {
                                            if !pack_name.is_empty() && vanilla_paths.iter().all(|x| &x.file_name().unwrap().to_string_lossy().to_string() != pack_name) {
                                                script_break.posible_pack = pack_name.to_owned();

                                                // This is only valid in newer games!!!
                                                let modd = game_config.mods().get(pack_name);
                                                script_break.posible_pack_mod = modd
                                                    .map(|modd| modd.name().to_string())
                                                    .unwrap_or_default();
                                                script_break.posible_pack_link = modd.and_then(|modd| modd.steam_id()
                                                    .as_ref()
                                                    .map(|id| format!("https://steamcommunity.com/sharedfiles/filedetails/?id={}", id))
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }

                                breaks.push(script_break);
                            }
                        }
                    }
                }

                // File-loading errors.
                /*
                [out] <2.8s>            Failed to load mod file [script\campaign\mod\test_errors_1.lua], error is: cannot open test_errors_1: No such file or directory. Will attempt to require() this file to generate a more meaningful error message:
                [out] <2.8s>                error loading module test_errors_1 from file test_errors_1:[string "script\campaign\mod\test_errors_1.lua"]:2: 'then' expected near 'aaaaa'
                [out] <2.8s>        Failed to load mod: [script\campaign\mod\test_errors_1.lua]


                [out] <2.8s>            Failed to execute loaded mod file [script\campaign\mod\test_error_3.lua], error is: [string "script\campaign\mod\test_error_3.lua"]:1: attempt to call global 'test_func' (a nil value)
                [out] <2.8s>        Failed to load mod: [script\campaign\mod\test_error_3.lua]

                 */
                let fail_load_errors = data.match_indices("Failed to load mod file").collect::<Vec<_>>();
                let fail_execute_errors = data.match_indices("Failed to execute loaded mod file").collect::<Vec<_>>();
                for (start_error, _) in fail_load_errors.into_iter().chain(fail_execute_errors.into_iter()) {

                    // For end we use the third out.
                    if let Some(end_error) = data[start_error..].find("Failed to load mod:") {
                        let message = data[start_error..start_error + end_error].to_owned();
                        let mut script_break = ScriptBreak {
                            full_log: message.to_owned(),
                            ..Default::default()
                        };

                        // PJ for some reason uses requires that fail when the CA loader does its thing. We need to ignore his mod.
                        if message.contains("Failed to load mod file [script\\campaign\\mod\\pj_") {
                            continue;
                        }

                        let start_path = "[string \"";
                        let end_path = "\"]:";
                        let mut paths = vec![];
                        for (start_path_pos, _) in message.match_indices(start_path) {
                            if let Some(end_path_pos) = message[start_path_pos + 9..].find(end_path) {
                                let path = message[start_path_pos + 9..start_path_pos + 9 + end_path_pos].replace("\\", "/");
                                paths.push(path);
                            }
                        }

                        // NOTE: pack finding only works if the pack that caused it is in the current run. Take that into account for tests.
                        for path in &paths {
                            if let Some(file) = pack.file(path, true) {
                                if let Some(pack_name) = file.container_name() {
                                    if !pack_name.is_empty() && vanilla_paths.iter().all(|x| &x.file_name().unwrap().to_string_lossy().to_string() != pack_name) {
                                        script_break.posible_pack = pack_name.to_owned();

                                        // This is only valid in newer games!!!
                                        let modd = game_config.mods().get(pack_name);
                                        script_break.posible_pack_mod = modd
                                            .map(|modd| modd.name().to_string())
                                            .unwrap_or_default();
                                        script_break.posible_pack_link = modd.and_then(|modd| modd.steam_id()
                                            .as_ref()
                                            .map(|id| format!("https://steamcommunity.com/sharedfiles/filedetails/?id={}", id))
                                        );
                                        break;
                                    }
                                }
                            }
                        }

                        breaks.push(script_break);
                    }
                }
            }
        }

        // If breaks are detected, show the dialog with them.
        if !breaks.is_empty() {

            // If breaks were found, load the UI Template.
            let template_path = if cfg!(debug_assertions) { LOG_ANALYSIS_VIEW_DEBUG } else { LOG_ANALYSIS_VIEW_RELEASE };
            let main_widget = load_template(self.main_window(), template_path)?;
            let dialog = main_widget.static_downcast::<QDialog>();

            let explanation_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "explanation_label")?;
            let explanation_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "explanation_groupbox")?;
            let breaks_table_view: QPtr<QTableView> = find_widget(&main_widget.static_upcast(), "breaks_table_view")?;
            explanation_label.set_text(&qtr("log_anaylis_explanation"));
            explanation_groupbox.set_title(&qtr("log_anaylis_explanation_title"));
            dialog.set_window_title(&qtr("log_anaylis_title"));

            let breaks_table_filter = QSortFilterProxyModel::new_1a(&breaks_table_view);
            let breaks_table_model = QStandardItemModel::new_1a(&breaks_table_filter);
            breaks_table_view.set_model(&breaks_table_filter);
            breaks_table_filter.set_source_model(&breaks_table_model);

            // Setup the table.
            breaks_table_model.set_column_count(2);

            let item_posible_pack = QStandardItem::from_q_string(&qtr("posible_pack"));
            let item_full_log = QStandardItem::from_q_string(&qtr("full_log"));

            breaks_table_view.horizontal_header().set_default_section_size(600);
            breaks_table_view.horizontal_header().set_stretch_last_section(true);

            breaks_table_model.set_horizontal_header_item(0, item_posible_pack.into_ptr());
            breaks_table_model.set_horizontal_header_item(1, item_full_log.into_ptr());

            html_item_delegate_safe(&breaks_table_view.static_upcast::<QObject>().as_ptr(), 0);

            // Load the data to the table.
            for script_break in &breaks {
                let row = QListOfQStandardItem::new();

                let item_pack = QStandardItem::new();
                let item_log = QStandardItem::new();

                item_pack.set_text(&QString::from_std_str(
                    match script_break.posible_pack_link() {
                        Some(link) => format!("<b>{}</b> (<i>{}</i>).<br/><br/>Link: <a src=\"{}\">{}</a>", script_break.posible_pack_mod(), script_break.posible_pack(), link, link),
                        None => script_break.posible_pack().to_string(),
                    }
                ));

                item_log.set_text(&QString::from_std_str(&script_break.full_log));

                row.append_q_standard_item(&item_pack.into_ptr().as_mut_raw_ptr());
                row.append_q_standard_item(&item_log.into_ptr().as_mut_raw_ptr());

                breaks_table_model.append_row_q_list_of_q_standard_item(row.into_ptr().as_ref().unwrap());
            }

            //breaks_table_view.resize_columns_to_contents();
            breaks_table_view.resize_rows_to_contents();

            dialog.set_modal(true);
            dialog.exec();
        }

        Ok(())
    }

    pub unsafe fn open_data_file_with_rpfm(&self) -> Result<()> {
        let tools = self.tools().read().unwrap();
        if let Some(tool) = tools.tools().iter().find(|tool| tool.path().ends_with("rpfm_ui.exe")) {
            if let Some(ref game_config) = *self.game_config().read().unwrap() {

                let game = self.game_selected().read().unwrap();
                let game_path = setting_path(game.key());
                if game_path.exists() && game_path.is_dir() {

                    let ca_packs = game.ca_packs_paths(&game_path)?;
                    let mut packs = vec![];
                    let mut files = vec![];

                    let selection = self.data_list_selection();
                    for selection in &selection {
                        if selection.column() == 0 {
                            files.push(<QPtr<QTreeView> as PackTree>::get_path_from_index(selection.as_ref(), self.data_list_ui().model()))
                        }

                        if selection.column() == 1 {

                            // About the packs, we search them by path in the
                            let pack = selection.data_0a().to_string().to_std_string();
                            if let Some(ca_pack) = ca_packs.iter().find(|ca_path| ca_path.file_name().unwrap().to_string_lossy() == pack) {
                                if !packs.contains(ca_pack) {
                                    packs.push(ca_pack.to_path_buf());
                                }
                            } else if let Some((_, modd)) = game_config.mods().iter()
                                .filter(|(_, modd)| !modd.paths().is_empty())
                                .find(|(_, modd)| modd.paths().first().unwrap().ends_with(&pack)) {

                                let path = modd.paths().first().unwrap();
                                if !packs.contains(path) {
                                    packs.push(path.to_path_buf());
                                }
                            }
                        }
                    }

                    let mut command = std::process::Command::new(tool.path().to_string_lossy().to_string());
                    for path in packs {
                        command.arg(path.to_string_lossy().to_string());
                    }

                    for path in files {
                        command.arg(path);
                    }

                    command.spawn()?;
                }
            }
        }

        Ok(())
    }

    fn save_load_order_file(file_path: &Path, game: &GameInfo, folder_list: &str, pack_list: &str) -> Result<()> {
        let mut file = BufWriter::new(File::create(file_path)?);

        // Napoleon, Empire and Shogun 2 require the user.script.txt or mod list file (for Shogun's latest update) to be in UTF-16 LE. What the actual fuck.
        if *game.raw_db_version() < 2 {
            file.write_string_u16(folder_list)?;
            file.write_string_u16(pack_list)?;
        } else {
            file.write_all(folder_list.as_bytes())?;
            file.write_all(pack_list.as_bytes())?;
        }

        file.flush().map_err(From::from)
    }
}
