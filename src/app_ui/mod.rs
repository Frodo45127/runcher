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
use qt_widgets::QPushButton;
use qt_widgets::QSplitter;
use qt_widgets::QTextEdit;
use qt_widgets::QWidget;

use qt_gui::QFont;
use qt_gui::QIcon;
use qt_gui::QStandardItem;

use qt_core::CheckState;
use qt_core::Orientation;
use qt_core::QBox;
use qt_core::QCoreApplication;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QSize;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::SlotNoArgs;

use cpp_core::CppBox;
use cpp_core::Ref;

use anyhow::{anyhow, Result};
use getset::Getters;
use itertools::Itertools;
use sha256::try_digest;

use std::collections::HashMap;
use std::fs::{copy, DirBuilder, File};
use std::io::{BufWriter, Read, Write};
#[cfg(target_os = "windows")] use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command as SystemCommand, exit};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::binary::WriteBytes;
use rpfm_lib::files::{EncodeableExtraData, pack::Pack, RFile};
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType, supported_games::*};
use rpfm_lib::integrations::log::*;
use rpfm_lib::schema::Schema;
use rpfm_lib::utils::files_from_subdir;

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::clone;
use rpfm_ui_common::locale::*;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::actions_ui::ActionsUI;
use crate::CENTRAL_COMMAND;
use crate::cli::Cli;
use crate::communications::*;
use crate::DARK_PALETTE;
use crate::ffi::*;
use crate::games::*;
use crate::mod_manager::{game_config::{GameConfig, DEFAULT_CATEGORY}, load_order::LoadOrder, mods::ShareableMod, profiles::Profile, saves::Save};
use crate::LIGHT_PALETTE;
use crate::LIGHT_STYLE_SHEET;
use crate::mod_list_ui::*;
use crate::pack_list_ui::PackListUI;
use crate::SCHEMA;
use crate::settings_ui::*;
use crate::SUPPORTED_GAMES;
use crate::updater_ui::*;

use self::slots::AppUISlots;

pub mod slots;

#[cfg(target_os = "windows")] const CREATE_NO_WINDOW: u32 = 0x08000000;
//const DETACHED_PROCESS: u32 = 0x00000008;

const LOAD_ORDER_STRING_VIEW_DEBUG: &str = "ui_templates/load_order_string_dialog.ui";
const LOAD_ORDER_STRING_VIEW_RELEASE: &str = "ui/load_order_string_dialog.ui";

pub const RESERVED_PACK_NAME: &str = "zzzzzzzzzzzzzzzzzzzzrun_you_fool_thron.pack";
pub const RESERVED_PACK_NAME_ALTERNATIVE: &str = "!!!!!!!!!!!!!!!!!!!!!run_you_fool_thron.pack";
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

    github_button: QBox<QPushButton>,
    discord_button: QBox<QPushButton>,
    patreon_button: QBox<QPushButton>,
    about_runcher_button: QBox<QPushButton>,
    check_updates_button: QBox<QPushButton>,

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
    // `Actions` section.
    //-------------------------------------------------------------------------------//
    actions_ui: Rc<ActionsUI>,

    //-------------------------------------------------------------------------------//
    // `Mod List` section.
    //-------------------------------------------------------------------------------//
    mod_list_ui: Rc<ModListUI>,

    //-------------------------------------------------------------------------------//
    // `Pack List` section.
    //-------------------------------------------------------------------------------//
    pack_list_ui: Rc<PackListUI>,

    //-------------------------------------------------------------------------------//
    // Extra stuff
    //-------------------------------------------------------------------------------//
    focused_widget: Rc<RwLock<Option<QPtr<QWidget>>>>,
    disabled_counter: Rc<RwLock<u32>>,

    game_config: Arc<RwLock<Option<GameConfig>>>,
    game_load_order: Arc<RwLock<LoadOrder>>,
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
    pub unsafe fn new() -> Result<Rc<Self>> {

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

        let app_ui = Rc::new(Self {

            //-------------------------------------------------------------------------------//
            // Main Window.
            //-------------------------------------------------------------------------------//
            main_window,

            github_button,
            discord_button,
            patreon_button,
            about_runcher_button,
            check_updates_button,

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
            game_load_order: Arc::new(RwLock::new(LoadOrder::default())),
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
        app_ui.reload_theme();

        // Apply last ui state.
        app_ui.main_window().restore_geometry(&setting_byte_array("geometry"));
        app_ui.main_window().restore_state_1a(&setting_byte_array("windowState"));

        // Apply the font.
        let font_name = setting_string("font_name");
        let font_size = setting_int("font_size");
        let font = QFont::from_q_string_int(&QString::from_std_str(font_name), font_size);
        QApplication::set_font_1a(&font);

        // Initialization logic. This takes care of parsing args for stuff like profile shortcuts,
        // or setting the game selected.
        //
        // NOTE: This exists if autostart param is passed, or if you pass invalid params,
        // so we don't need to load anything regarthing the UI.
        match Cli::parse_args(&app_ui) {
            Ok(autostart) => if autostart {
                exit(0);
            },
            Err(error) => {
                show_dialog(app_ui.main_window(), error, false);
                exit(1);
            },
        }

        // Check for updates.
        UpdaterUI::new_with_precheck(&app_ui)?;

        Ok(app_ui)
    }

    pub unsafe fn set_connections(&self, slots: &AppUISlots) {
        self.actions_ui().play_button().released().connect(slots.launch_game());
        self.actions_ui().enable_logging_checkbox().toggled().connect(slots.toggle_logging());
        self.actions_ui().enable_skip_intro_checkbox().toggled().connect(slots.toggle_skip_intros());
        self.actions_ui().merge_all_mods_checkbox().toggled().connect(slots.toggle_merge_all_mods());
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

        self.about_runcher_button().released().connect(slots.about_runcher());
        self.check_updates_button().released().connect(slots.check_updates());

        self.github_button().released().connect(slots.github_link());
        self.discord_button().released().connect(slots.discord_link());
        self.patreon_button().released().connect(slots.patreon_link());

        self.mod_list_ui().model().item_changed().connect(slots.update_pack_list());
        self.mod_list_ui().context_menu().about_to_show().connect(slots.mod_list_context_menu_open());
        self.mod_list_ui().enable_selected().triggered().connect(slots.enable_selected());
        self.mod_list_ui().disable_selected().triggered().connect(slots.disable_selected());
        self.mod_list_ui().category_new().triggered().connect(slots.category_create());
        self.mod_list_ui().category_delete().triggered().connect(slots.category_delete());
        self.mod_list_ui().category_rename().triggered().connect(slots.category_rename());
        draggable_tree_view_drop_signal(self.mod_list_ui().tree_view().static_upcast()).connect(slots.category_move());

        self.pack_list_ui().automatic_order_button().toggled().connect(slots.pack_toggle_auto_sorting());
        draggable_tree_view_drop_signal(self.pack_list_ui().tree_view().static_upcast()).connect(slots.pack_move());
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

    pub unsafe fn change_game_selected(&self, reload_same_game: bool, skip_network_update: bool) -> Result<()> {

        // Get the new `Game Selected` and clean his name up, so it ends up like "x_y".
        let mut new_game_selected = self.game_selected_group.checked_action().text().to_std_string();
        if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
        let new_game_selected = new_game_selected.replace(' ', "_").to_lowercase();

        // If the game changed or we're initializing the program, change the game selected.
        //
        // This works because by default, the initially stored game selected is arena, and that one can never set manually.
        if reload_same_game || new_game_selected != self.game_selected().read().unwrap().key() {
            self.load_data(&new_game_selected, skip_network_update)?;
        }

        Ok(())
    }

    pub unsafe fn load_data(&self, game: &str, skip_network_update: bool) -> Result<()> {

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
                let _ = Profile::update(&self.game_config().read().unwrap().clone().unwrap(), &game);

                // Load the profile's list.
                match Profile::profiles_for_game(game) {
                    Ok(profiles) => *self.game_profiles().write().unwrap() = profiles,
                    Err(error) => show_dialog(self.main_window(), format!("Error loading profiles: {}", error), false),
                }

                self.actions_ui().profile_model().clear();
                for profile in self.game_profiles().read().unwrap().keys().sorted() {
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
                if let Err(error) = self.load_mods_to_ui(game, &game_path, skip_network_update) {
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

    pub unsafe fn load_mods_to_ui(&self, game: &GameInfo, game_path: &Path, skip_network_update: bool) -> Result<()> {
        let mut mods = self.game_config().write().unwrap();
        if let Some(ref mut mods) = *mods {
            let mut load_order = self.game_load_order().write().unwrap();
            mods.update_mod_list(game, game_path, &mut load_order, skip_network_update)?;

            self.mod_list_ui().load(mods)?;
            self.pack_list_ui().load(mods, game, game_path, &load_order)?;
        }

        Ok(())
    }

    pub unsafe fn open_settings(&self) {
        let game_selected = self.game_selected().read().unwrap();
        let game_key = game_selected.key();
        let game_path_old = setting_path(game_key);
        let dark_theme_old = setting_bool("dark_mode");
        let font_name_old = setting_string("font_name");
        let font_size_old = setting_int("font_size");

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
        let game = self.game_selected().read().unwrap();
        let game_path = setting_path(game.key());
        let data_path = game.data_path(&game_path)?;

        // We only use the reserved pack if we need to.
        if (self.actions_ui().enable_logging_checkbox().is_enabled() && self.actions_ui().enable_logging_checkbox().is_checked()) ||
            (self.actions_ui().enable_skip_intro_checkbox().is_enabled() && self.actions_ui().enable_skip_intro_checkbox().is_checked()) ||
            (self.actions_ui().enable_translations_combobox().is_enabled() && self.actions_ui().enable_translations_combobox().current_index() != 0) ||
            (self.actions_ui().unit_multiplier_spinbox().is_enabled() && self.actions_ui().unit_multiplier_spinbox().value() != 1.00) {

            // We need to use an alternative name for Shogun 2, Rome 2, Attila and Thrones because their load order logic for movie packs seems... either different or broken.
            let reserved_pack_name = if game.key() == KEY_SHOGUN_2 || game.key() == KEY_ROME_2 || game.key() == KEY_ATTILA || game.key() == KEY_THRONES_OF_BRITANNIA { RESERVED_PACK_NAME_ALTERNATIVE } else { RESERVED_PACK_NAME };

            // Support for add_working_directory seems to be only present in rome 2 and newer games. For older games, we drop the pack into /data.
            let temp_path = if *game.raw_db_version() >= 2 {
                let temp_packs_folder = temp_packs_folder(&game)?;
                let temp_path = temp_packs_folder.join(reserved_pack_name);
                folder_list.push_str(&format!("add_working_directory \"{}\";\n", temp_packs_folder.to_string_lossy()));
                temp_path
            } else {
                data_path.join(reserved_pack_name)
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
                    let mut reserved_pack = Pack::read_and_merge(&pack_paths, true, false)?;
                    let pack_version = game.pfh_version_by_file_type(PFHFileType::Mod);
                    reserved_pack.set_pfh_version(pack_version);

                    let mut encode_data = EncodeableExtraData::default();
                    encode_data.set_nullify_dates(true);

                    reserved_pack.save(Some(&temp_path), &game, &Some(encode_data))?;
                }
            } else {
                return Err(anyhow!(tr("game_config_error")));
            }
        }

        // Otherwise, just add the packs from the load order to the text file.
        else {

            if let Some(ref game_config) = *self.game_config().read().unwrap() {
                let load_order = self.game_load_order().read().unwrap();
                load_order.build_load_order_string(game_config, &game, &data_path, &mut pack_list, &mut folder_list);
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
                                metadata.len() > 1_000_000
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
                    load_order.update(game_config);

                    // Reload the pack list.
                    let game_info = self.game_selected().read().unwrap();

                    if !is_autostart {
                        if let Err(error) = load_order.save(&game_info) {
                            show_dialog(self.main_window(), error, false);
                        }
                    }

                    let game_path = setting_path(game_info.key());
                    if let Err(error) = self.pack_list_ui().load(game_config, &game_info, &game_path, &load_order) {
                        show_dialog(self.main_window(), error, false);
                    }

                    if !is_autostart {
                        if let Err(error) = game_config.save(&game_info) {
                            show_dialog(self.main_window(), error, false);
                        }
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

            // TODO: This is wrong!!!! the load order from shared lists needs a rework.
            let game = self.game_selected().read().unwrap();
            let game_path = setting_path(game.key());
            let load_order = self.game_load_order().read().unwrap();
            self.mod_list_ui().load(game_config)?;
            self.pack_list_ui().load(game_config, &game, &game_path, &load_order)?;

            game_config.save(&game)?;
        }

        Ok(())
    }

    pub unsafe fn batch_toggle_selected_mods(&self, toggle: bool) {

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
            let mut load_order = self.game_load_order().write().unwrap();
            load_order.update(game_config);

            if let Err(error) = load_order.save(&game_info) {
                show_dialog(self.main_window(), error, false);
            }

            if let Err(error) = self.pack_list_ui().load(game_config, &game_info, &game_path, &load_order) {
                show_dialog(self.main_window(), error, false);
            }

            if let Err(error) = game_config.save(&game_info) {
                show_dialog(self.main_window(), error, false);
            }
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

    pub unsafe fn move_category(&self, dest_parent: Ref<QModelIndex>, dest_row: i32) -> Result<()> {

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

                let category_index_logical = self.mod_list_ui().filter().map_to_source(category_index_visual);
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
                if let Some(item) = app_ui.mod_list_ui().category_item(category) {
                    let action = app_ui.mod_list_ui().categories_send_to_menu().add_action_q_string(&QString::from_std_str(category));
                    let slot = SlotNoArgs::new(app_ui.mod_list_ui().categories_send_to_menu(), clone!(
                        category,
                        app_ui => move || {
                            let mut selection = app_ui.mod_list_selection();
                            selection.reverse();

                            for mod_item in &selection {
                                let current_cat = mod_item.parent();
                                let mod_id = mod_item.data_1a(VALUE_MOD_ID).to_string().to_std_string();
                                let taken = app_ui.mod_list_ui().model().item_from_index(&current_cat).take_row(mod_item.row()).into_ptr();
                                item.append_row_q_list_of_q_standard_item(taken.as_ref().unwrap());

                                if let Some(ref mut game_config) = *app_ui.game_config().write().unwrap() {
                                    let curr_cat = game_config.category_for_mod(&mod_id);
                                    if let Some(ids) = game_config.categories_mut().get_mut(&curr_cat) {
                                        if let Some(pos) = ids.iter().position(|x| x == &mod_id) {
                                            ids.remove(pos);
                                        }
                                    }

                                    if let Some(ids) = game_config.categories_mut().get_mut(&category) {
                                        ids.push(mod_id);
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
            .filter(|(index, mod_id)| (index < &(new_position as usize) && packs_to_move.contains(&mod_id)))
            .count();

        load_order.mods_mut().retain(|mod_id| !packs_to_move.contains(&mod_id));
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
                item.set_data_2a(&QVariant::from_int(row as i32), 2);
            }
        }

        Ok(())

    }
}
