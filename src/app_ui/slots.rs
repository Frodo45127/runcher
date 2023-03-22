//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QMessageBox;

use qt_gui::SlotOfQStandardItem;

use qt_core::QBox;
use qt_core::SlotNoArgs;

use sha256::try_digest;

use std::cmp::Reverse;
use std::sync::Arc;

use rpfm_ui_common::clone;

use crate::mod_list_ui::VALUE_MOD_ID;
use crate::VERSION;
use crate::VERSION_SUBTITLE;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct AppUISlots {
    launch_game: QBox<SlotNoArgs>,
    open_settings: QBox<SlotNoArgs>,
    open_game_root_folder: QBox<SlotNoArgs>,
    open_game_data_folder: QBox<SlotNoArgs>,
    open_game_content_folder: QBox<SlotNoArgs>,
    open_runcher_config_folder: QBox<SlotNoArgs>,
    open_runcher_error_folder: QBox<SlotNoArgs>,
    change_game_selected: QBox<SlotNoArgs>,

    update_pack_list: QBox<SlotOfQStandardItem>,

    about_qt: QBox<SlotNoArgs>,
    about_runcher: QBox<SlotNoArgs>,
    check_updates: QBox<SlotNoArgs>,

    copy_load_order: QBox<SlotNoArgs>,
    paste_load_order: QBox<SlotNoArgs>,
    reload: QBox<SlotNoArgs>,
    load_profile: QBox<SlotNoArgs>,
    save_profile: QBox<SlotNoArgs>,

    category_delete: QBox<SlotNoArgs>,
    mod_list_context_menu_open: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AppUISlots {
    pub unsafe fn new(view: &Arc<AppUI>) -> Self {

        let launch_game = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                if let Err(error) = view.launch_game() {
                    show_dialog(view.main_window(), error, false);
                }
            }
        ));

        let open_settings = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
            view.open_settings();
        }));

        let open_game_root_folder = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
            let game = view.game_selected().read().unwrap();
            let game_path = setting_string(game.game_key_name());
            if !game_path.is_empty() {
                let _ = open::that(game_path);
            } else {
                show_dialog(view.main_window(), "Runcher cannot open that folder (maybe it doesn't exists/is misconfigured?).", false);
            }
        }));

        let open_game_data_folder = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
            let game = view.game_selected().read().unwrap();
            if let Ok(game_path) = game.data_path(&setting_path(game.game_key_name())) {
                let _ = open::that(game_path);
            } else {
                show_dialog(view.main_window(), "Runcher cannot open that folder (maybe it doesn't exists/is misconfigured?).", false);
            }
        }));

        let open_game_content_folder = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
            let game = view.game_selected().read().unwrap();
            if let Ok(game_path) = game.content_path(&setting_path(game.game_key_name())) {
                let _ = open::that(game_path);
            } else {
                show_dialog(view.main_window(), "Runcher cannot open that folder (maybe it doesn't exists/is misconfigured?).", false);
            }
        }));

        let open_runcher_config_folder = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
            if let Ok(path) = config_path() {
                let _ = open::that(path);
            } else {
                show_dialog(view.main_window(), "Runcher cannot open that folder (maybe it doesn't exists/is misconfigured?).", false);
            }
        }));

        let open_runcher_error_folder = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
            if let Ok(path) = error_path() {
                let _ = open::that(path);
            } else {
                show_dialog(view.main_window(), "Runcher cannot open that folder (maybe it doesn't exists/is misconfigured?).", false);
            }
        }));

        let change_game_selected = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                if let Err(error) = view.change_game_selected() {
                    show_dialog(view.main_window(), error, false);
                }
            }
        ));

        let update_pack_list = SlotOfQStandardItem::new(&view.main_window, clone!(
            view => move |item| {
            if item.column() == 0 {
                if let Some(ref mut game_config) = *view.game_config().write().unwrap() {
                    let mod_id = item.data_1a(VALUE_MOD_ID).to_string().to_std_string();

                    // Update the mod's status.
                    if let Some(modd) = game_config.mods_mut().get_mut(&mod_id) {
                        modd.set_enabled(item.check_state() == CheckState::Checked);
                    }

                    // Reload the pack view.
                    let game_info = view.game_selected().read().unwrap();
                    let game_path = setting_path(game_info.game_key_name());
                    if let Err(error) = view.pack_list_ui().load(game_config, &game_info, &game_path) {
                        show_dialog(view.main_window(), error, false);
                    }

                    let game_info = view.game_selected().read().unwrap();
                    if let Err(error) = game_config.save(&game_info) {
                        show_dialog(view.main_window(), error, false);
                    }
                }
            }
        }));

        let about_qt = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                QMessageBox::about_qt_1a(&view.main_window);
            }
        ));

        let about_runcher = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                QMessageBox::about(
                    &view.main_window,
                    &qtr("about_runcher"),

                    // NOTE: This one is hardcoded, because I don't want people attributing themselves the program in the translations.
                    &QString::from_std_str(format!(
                        "<table>
                            <tr>
                                <td><h2><b>Runcher</b></h2></td>
                            </tr>
                            <tr>
                                <td>{} {} Patch</td>
                            </tr>
                        </table>

                        <p><b>Rusted Launcher</b> (a.k.a. Runcher) is a mod manager/launcher for modern Total War Games.</p>
                        <p>This program is <b>open-source</b>, under MIT License. You can always get the last version (or collaborate) here:</p>
                        <a href=\"https://github.com/Frodo45127/runcher\">https://github.com/Frodo45127/runcher</a>
                        <p>This program is also <b>free</b> (if you paid for this, sorry, but you got scammed), but if you want to help with money, here is <b>RPFM's Patreon</b>:</p>
                        <a href=\"https://www.patreon.com/RPFM\">https://www.patreon.com/RPFM</a>

                        <h3>Credits</h3>
                        <ul style=\"list-style-type: disc\">
                            <li>Created and Programmed by: <b>Frodo45127</b>.</li>
                        </ul>
                        ", &VERSION, &VERSION_SUBTITLE)
                    )
                );
            }
        ));

        let check_updates = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                view.check_updates(true);
            }
        ));

        let copy_load_order = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                if let Some(ref game_config) = *view.game_config().read().unwrap() {
                    view.toggle_main_window(false);

                    let receiver = CENTRAL_COMMAND.send_background(Command::GetStringFromLoadOrder(game_config.clone()));
                    let response = CENTRAL_COMMAND.recv_try(&receiver);
                    match response {
                        Response::String(response) => {
                            if let Err(error) = view.load_order_string_dialog(Some(response)) {
                                show_dialog(view.main_window(), error, false)
                            }
                        }
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }

                    view.toggle_main_window(true);
                }
            }
        ));

        let paste_load_order = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                match view.load_order_string_dialog(None) {
                    Ok(string) => if let Some(string) = string {
                        view.toggle_main_window(false);

                        let receiver = CENTRAL_COMMAND.send_background(Command::GetLoadOrderFromString(string));
                        let response = CENTRAL_COMMAND.recv_try(&receiver);
                        match response {
                            Response::VecShareableMods(response) => {
                                if let Some(ref mut game_config) = *view.game_config().write().unwrap() {
                                    let mut missing = vec![];
                                    let mut wrong_hash = vec![];
                                    for modd in &response {
                                        match game_config.mods_mut().get_mut(modd.id()) {
                                            Some(modd_local) => {
                                                let current_hash = try_digest(modd_local.paths()[0].as_path()).unwrap();
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
                                        show_dialog(view.main_window(), message, false);
                                    }

                                    let game = view.game_selected().read().unwrap();
                                    let game_path = setting_path(game.game_key_name());
                                    view.mod_list_ui().load(game_config).unwrap();
                                    view.pack_list_ui().load(game_config, &game, &game_path).unwrap();

                                    if let Err(error) = game_config.save(&game) {
                                        show_dialog(view.main_window(), error, false);
                                    }
                                }
                            }
                            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                        }

                        view.toggle_main_window(true);
                    }
                    Err(error) => show_dialog(view.main_window(), error, false),
                }
            }
        ));

        let reload = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {

                // We just re-use the game selected logic
                if let Err(error) = view.change_game_selected() {
                    show_dialog(view.main_window(), error, false);
                }
            }
        ));

        let load_profile = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                if let Err(error) = view.load_profile() {
                    show_dialog(view.main_window(), error, false);
                }
            }
        ));

        let save_profile = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                if let Err(error) = view.save_profile() {
                    show_dialog(view.main_window(), error, false);
                }
            }
        ));

        let category_delete = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                let mut selection = view.mod_list_selection();
                selection.sort_by_key(|b| Reverse(b.row()));

                if selection.iter().any(|index| index.data_1a(2).to_string().to_std_string() == "Unassigned") {
                    return;
                }

                for cat_to_delete in &selection {
                    let mods_to_reassign = (0..view.mod_list_ui().model().row_count_1a(cat_to_delete))
                        .map(|index| cat_to_delete.child(index, 0).data_1a(VALUE_MOD_ID).to_string().to_std_string())
                        .collect::<Vec<_>>();

                    if let Some(ref mut game_config) = *view.game_config().write().unwrap() {
                        game_config.mods_mut()
                            .iter_mut()
                            .for_each(|(id, modd)| if mods_to_reassign.contains(id) {
                                modd.set_category(None);
                            });
                    }

                    // Find the unassigned category.
                    let mut unassigned_item = None;
                    let unassigned = QString::from_std_str("Unassigned");
                    for index in 0..view.mod_list_ui().model().row_count_0a() {
                        let item = view.mod_list_ui().model().item_1a(index);
                        if !item.is_null() && item.text().compare_q_string(&unassigned) == 0 {
                            unassigned_item = Some(item);
                            break;
                        }
                    }

                    if let Some(unassigned_item) = unassigned_item {
                        let cat_item = view.mod_list_ui().model().item_from_index(cat_to_delete);
                        for index in (0..view.mod_list_ui().model().row_count_1a(cat_to_delete)).rev() {
                            let taken = cat_item.take_row(index).into_ptr();
                            unassigned_item.append_row_q_list_of_q_standard_item(taken.as_ref().unwrap());
                        }
                    }

                    view.mod_list_ui().model().remove_row_1a(cat_to_delete.row());
                }

                let game_info = view.game_selected().read().unwrap();
                if let Some(ref mut game_config) = *view.game_config().write().unwrap() {
                    if let Err(error) = game_config.save(&game_info) {
                        show_dialog(view.main_window(), error, false);
                    }
                }
            }
        ));

        let mod_list_context_menu_open = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
                view.mod_list_ui().categories_send_to_menu().clear();
                let categories = view.mod_list_ui().categories();
                for category in &categories {

                    let item = view.mod_list_ui().category_item(category);
                    if let Some(item) = item {
                        let action = view.mod_list_ui().categories_send_to_menu().add_action_q_string(&QString::from_std_str(category));
                        let slot = SlotNoArgs::new(view.mod_list_ui().categories_send_to_menu(), clone!(
                            category,
                            view => move || {
                                let mut selection = view.mod_list_selection();
                                selection.sort_by_key(|b| Reverse(b.row()));

                                for mod_item in &selection {
                                    let current_cat = mod_item.parent();
                                    let mod_id = mod_item.data_1a(VALUE_MOD_ID).to_string().to_std_string();
                                    let taken = view.mod_list_ui().model().item_from_index(&current_cat).take_row(mod_item.row()).into_ptr();
                                    item.append_row_q_list_of_q_standard_item(taken.as_ref().unwrap());

                                    if let Some(ref mut game_config) = *view.game_config().write().unwrap() {
                                        if let Some(ref mut modd) = game_config.mods_mut().get_mut(&mod_id) {
                                            modd.set_category(Some(category.to_string()));
                                        }

                                        let game_info = view.game_selected().read().unwrap();
                                        if let Err(error) = game_config.save(&game_info) {
                                            show_dialog(view.main_window(), error, false);
                                        }
                                    }
                                }
                            }
                        ));

                        action.triggered().connect(&slot);
                    }
                }
            }
        ));

        Self {
            launch_game,
            open_settings,
            open_game_root_folder,
            open_game_data_folder,
            open_game_content_folder,
            open_runcher_config_folder,
            open_runcher_error_folder,
            change_game_selected,

            update_pack_list,

            about_qt,
            about_runcher,
            check_updates,

            copy_load_order,
            paste_load_order,
            reload,

            load_profile,
            save_profile,

            category_delete,
            mod_list_context_menu_open,
        }
    }
}
