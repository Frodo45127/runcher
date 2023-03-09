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

use qt_gui::QIcon;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;

use anyhow::{anyhow, Result};
use getset::Getters;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::games::{GameInfo, supported_games::*};
use rpfm_lib::integrations::log::*;

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::actions_ui::ActionsUI;
use crate::mod_list_ui::ModListUI;
use crate::pack_list_ui::PackListUI;
use crate::settings_ui::SettingsUI;
use crate::settings_ui::init_settings;
use crate::SUPPORTED_GAMES;

use self::slots::AppUISlots;

pub mod slots;

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
    game_selected: Rc<RwLock<GameInfo>>,
    focused_widget: Rc<RwLock<Option<QPtr<QWidget>>>>,
    disabled_counter: Rc<RwLock<u32>>,
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
            game_selected: Rc::new(RwLock::new(SUPPORTED_GAMES.game("warhammer_2").unwrap().clone())),
            focused_widget: Rc::new(RwLock::new(None)),
            disabled_counter: Rc::new(RwLock::new(0)),
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

        /*
        // If we don't have any path in the settings, launch the settings window while disabling the main window.
        // We NEED at least one correct path for this to work. If no path is set, we close the application.
        let mut game_path_set = false;
        for game in SUPPORTED_GAMES.games() {
            let setting_key = format!("game_path_{}", game.game_key_name());
            let setting = setting_path(&setting_key);
            if setting.is_dir() {
                game_path_set = true;
                break;
            }
        }

        if !game_path_set {
            SettingsUI::new(app_ui.main_window());
        }*/

        log_to_status_bar(app_ui.main_window().status_bar(), "Initializing, please wait...");

        let game = SUPPORTED_GAMES.game("warhammer_2").unwrap();
        let game_path = std::path::PathBuf::from("/home/frodo45127/test/warhammer_2/");

        app_ui.pack_list_ui().load(game, &game_path);

        Ok(app_ui)
    }

    pub unsafe fn set_connections(&self, slots: &AppUISlots) {
        self.actions_ui().play_button().released().connect(slots.launch_game());
        self.actions_ui().settings_button().released().connect(slots.open_settings());
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

    pub unsafe fn set_game_selected(&self, game: &str) -> Result<()> {

        // We may receive invalid games here, so rule out the invalid ones.
        match SUPPORTED_GAMES.game(game) {
            Some(game) => {
                *self.game_selected().write().unwrap() = game.clone();
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
}
