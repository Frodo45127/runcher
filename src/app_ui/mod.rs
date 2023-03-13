//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QAction;
use qt_widgets::QActionGroup;
use qt_widgets::QApplication;
use qt_widgets::QMainWindow;
use qt_widgets::QWidget;

use qt_core::CheckState;
use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;

use anyhow::{anyhow, Result};
use getset::Getters;

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::files::pack::Pack;
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType, supported_games::*};
use rpfm_lib::integrations::log::*;

use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::actions_ui::ActionsUI;
use crate::integrations::{GameConfig, Mod};
use crate::mod_list_ui::ModListUI;
use crate::pack_list_ui::PackListUI;
use crate::settings_ui::SettingsUI;
use crate::settings_ui::init_settings;
use crate::SUPPORTED_GAMES;

use self::slots::AppUISlots;

pub mod slots;

const CREATE_NO_WINDOW: u32 = 0x08000000;
//const DETACHED_PROCESS: u32 = 0x00000008;

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

    //-------------------------------------------------------------------------------//
    // `Actions` section.
    //-------------------------------------------------------------------------------//
    actions_ui: ActionsUI,

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

    // Game selected. Unlike RPFM, here it's not a global.
    game_selected: Rc<RwLock<GameInfo>>,

    // List of Packs, split by type, available for the game selected.
    //
    // This includes vanilla and disabled packs, from data and from content, split by path.
    //
    // If there is a collision between content and data, only data is used.
    packs: Arc<RwLock<BTreeMap<PFHFileType, HashMap<String, Pack>>>>,

    // List of Mods found for the game selected.
    //
    // These may be individual Packs, or group of Packs, of type `Mod`, separated by category.
    mods: Arc<RwLock<HashMap<String, Vec<Mod>>>>,

    // Mod load order, by path. Only includes mods, as the rest of the Packs are always in the same order.
    load_order: Arc<RwLock<Vec<String>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AppUI {

    /// This function creates an entire `AppUI` struct. Used to create the entire UI at start.
    pub unsafe fn new() -> Result<Arc<Self>> {

        // Initialize and configure the main window.
        let main_window = QMainWindow::new_0a();
        let widget = QWidget::new_1a(&main_window);
        let _ = create_grid_layout(widget.static_upcast());
        main_window.set_central_widget(&widget);
        main_window.resize_2a(1300, 800);
        //QApplication::set_window_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/rpfm.png", ASSETS_PATH.to_string_lossy()))));

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
        let about_about_qt = menu_bar_about.add_action_q_string(&qtr("About QT"));
        let about_about_runcher = menu_bar_about.add_action_q_string(&qtr("About Runcher"));

        //-------------------------------------------------------------------------------//
        // `Actions` section.
        //-------------------------------------------------------------------------------//
        let actions_ui = ActionsUI::new(&main_window)?;

        //-------------------------------------------------------------------------------//
        // `Mod List` section.
        //-------------------------------------------------------------------------------//
        let mod_list_ui = ModListUI::new(&main_window)?;

        //-------------------------------------------------------------------------------//
        // `Pack List` section.
        //-------------------------------------------------------------------------------//
        let pack_list_ui = PackListUI::new(&main_window)?;

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

            // NOTE: This loads arena on purpose, so ANY game selected triggers a game change properly.
            game_selected: Rc::new(RwLock::new(SUPPORTED_GAMES.game("arena").unwrap().clone())),
            packs: Arc::new(RwLock::new(BTreeMap::new())),
            mods: Arc::new(RwLock::new(HashMap::new())),
            load_order: Arc::new(RwLock::new(Vec::new())),
        });

        let slots = AppUISlots::new(&app_ui);
        app_ui.set_connections(&slots);

        // Initialize settings.
        init_settings(&app_ui.main_window().static_upcast());

        // Apply last ui state.
        app_ui.main_window().restore_geometry(&setting_byte_array("geometry"));
        app_ui.main_window().restore_state_1a(&setting_byte_array("windowState"));

        // Show the Main Window.
        app_ui.main_window().show();
        log_to_status_bar(app_ui.main_window().status_bar(), "Initializing, please wait...");

        // Set the game selected based on the default game.
        //
        // Note: set_checked does *NOT* trigger the slot for changing game selected. We need to trigger that one manually.
        // TODO: Allow to provide said game through an argument.
        let default_game = setting_string("default_game");
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
        app_ui.set_game_selected(&default_game)?;

        Ok(app_ui)
    }

    pub unsafe fn set_connections(&self, slots: &AppUISlots) {
        self.actions_ui().play_button().released().connect(slots.launch_game());
        self.actions_ui().settings_button().released().connect(slots.open_settings());

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

        self.mod_list_ui().model().item_changed().connect(slots.update_pack_list());
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

    pub unsafe fn change_game_selected(&self) -> Result<()> {

        // Get the new `Game Selected` and clean his name up, so it ends up like "x_y".
        let mut new_game_selected = self.game_selected_group.checked_action().text().to_std_string();
        if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
        let new_game_selected = new_game_selected.replace(' ', "_").to_lowercase();

        // If the game changed or we're initializing the program, change the game selected.
        if new_game_selected != self.game_selected().read().unwrap().game_key_name() {
            self.set_game_selected(&new_game_selected)?;
        }

        Ok(())
    }

    pub unsafe fn set_game_selected(&self, game: &str) -> Result<()> {

        // We may receive invalid games here, so rule out the invalid ones.
        match SUPPORTED_GAMES.game(game) {
            Some(game) => {
                *self.game_selected().write().unwrap() = game.clone();

                // Load the game's config.
                *self.game_config().write().unwrap() = Some(GameConfig::load(&game, true)?);

                // If we don't have a path in the settings for the game, disable the play button.
                let game_path = setting_string(game.game_key_name());
                self.actions_ui().play_button().set_enabled(!game_path.is_empty());

                // If we have a path, load all the mods to the UI.
                if !game_path.is_empty() {
                    let game_path = PathBuf::from(game_path);
                    let data_paths = game.data_packs_paths(&game_path);
                    let content_paths = game.content_packs_paths(&game_path);

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
                                            }
                                            None => {
                                                let mut modd = Mod::default();
                                                modd.set_name(pack_name.to_owned());
                                                modd.set_paths(vec![path.to_path_buf()]);
                                                mods.mods_mut().insert(pack_name, modd);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let mods = self.game_config().read().unwrap();
                    if let Some(ref mods) = *mods {
                        self.mod_list_ui().load(&mods)?;
                        self.pack_list_ui().load(&mods)?;
                    }
                }

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
        let pack_list = (0..self.pack_list_ui().model().row_count_0a())
            .filter_map(|index| {
                let item = self.pack_list_ui().model().item_1a(index);
                Some(format!("mod \"{}\";", item.text().to_std_string()))
                /*
                if item.is_checkable() && item.check_state() == CheckState::Checked {
                    Some(format!("mod \"{}\"", item.text().to_std_string()))
                } else {
                    None
                }*/
            })
            .collect::<Vec<_>>()
            .join("\n");

        let game = self.game_selected().read().unwrap();
        let game_path = setting_path(&game.game_key_name());
        let file_path = game_path.join("mod_list.txt");

        let mut file = BufWriter::new(File::create(&file_path)?);
        file.write_all(pack_list.as_bytes())?;
        file.flush()?;

        let exec_game = game.executable_path(&game_path).unwrap();

        if cfg!(target_os = "windows") {
            let mut command = Command::new("cmd");
            command.arg("/C");
            command.arg("start");
            command.arg("/d");
            command.arg(game_path.to_string_lossy().replace("\\", "/"));
            command.arg(exec_game.file_name().unwrap().to_string_lossy().to_string());
            command.arg("mod_list.txt;");

            // This disables the terminal when executing the command.
            command.creation_flags(CREATE_NO_WINDOW);
            command.spawn()?;

            Ok(())
        } else if cfg!(target_os = "linux") {

            Ok(())
        } else {
            Err(anyhow!("Unsupported OS."))
        }
    }
}
