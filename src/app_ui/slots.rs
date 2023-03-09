//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::CheckState;
use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::io::{BufWriter, Write};
use std::fs::File;
use std::sync::Arc;

use rpfm_ui_common::clone;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct AppUISlots {
    launch_game: QBox<SlotNoArgs>,
    open_settings: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AppUISlots {
    pub unsafe fn new(view: &Arc<AppUI>) -> Self {

        let launch_game = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {

            let pack_list = (0..view.pack_list_ui().model().row_count_0a())
                .filter_map(|index| {
                    let item = view.pack_list_ui().model().item_1a(index);
                    if item.is_checkable() && item.check_state() == CheckState::Checked {
                        Some(format!("mod \"{}\"", item.text().to_std_string()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            let game = view.game_selected().read().unwrap();
            let game_path = setting_path(&format!("game_path_{}", game.game_key_name()));
            let file_path = game_path.join("used_mods.txt");

            dbg!(&pack_list);
            /*
            let mut file = BufWriter::new(File::create(file_path).unwrap());
            file.write_all(pack_list.as_bytes()).unwrap();

            let exec_game = game.executable_path(&game_path).unwrap();*/
        }));

        let open_settings = SlotNoArgs::new(&view.main_window, clone!(
            view => move || {
            view.open_settings();
        }));

        Self {
            launch_game,
            open_settings
        }
    }
}
