//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QApplication;

use qt_core::QBox;
use qt_core::SlotNoArgs;

use getset::*;

use std::env::current_exe;
use std::process::{Command as SystemCommand, exit};
use std::rc::Rc;

use rpfm_ui_common::clone;
use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::utils::show_dialog;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;

use super::UpdaterUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct UpdaterUISlots {
    update_program: QBox<SlotNoArgs>,
    update_schemas: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl UpdaterUISlots {

    pub unsafe fn new(ui: &Rc<UpdaterUI>, app_ui: &Rc<AppUI>) -> Self {
        let update_program = SlotNoArgs::new(ui.main_widget(), clone!(
            ui => move || {
                let receiver = CENTRAL_COMMAND.send_background(Command::UpdateMainProgram);
                ui.update_program_button.set_text(&qtr("updater_update_schemas_updating"));
                ui.update_program_button.set_enabled(false);

                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::Success => {
                        ui.update_program_button.set_text(&qtr("updater_update_program_updated"));

                        // Re-enable the button so it can be used to restart the program.
                        ui.update_program_button.set_enabled(true);
                        ui.update_program_button.released().connect(&SlotNoArgs::new(ui.main_widget(), move || {

                            // Make sure we close both threads and the window. In windows the main window doesn't get closed for some reason.
                            CENTRAL_COMMAND.send_background(Command::Exit);
                            CENTRAL_COMMAND.send_network(Command::Exit);
                            QApplication::close_all_windows();

                            let exe_path = current_exe().unwrap();
                            SystemCommand::new(exe_path).spawn().unwrap();
                            exit(10);
                        }));
                    },
                    Response::Error(error) => {
                        show_dialog(ui.dialog(), error, false);
                        ui.update_program_button.set_text(&qtr("updater_update_program_error"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            }
        ));

        let update_schemas = SlotNoArgs::new(ui.main_widget(), clone!(
            app_ui,
            ui => move || {
                let receiver = CENTRAL_COMMAND.send_background(Command::UpdateSchemas(app_ui.game_selected().read().unwrap().schema_file_name().to_owned()));
                ui.update_schemas_button.set_text(&qtr("updater_update_schemas_updating"));
                ui.update_schemas_button.set_enabled(false);

                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::Success => {

                        // We need to reload the game in question, so stuff that depends on schemas existing actually works.
                        if let Err(error) = app_ui.change_game_selected(true, true) {
                            show_dialog(ui.dialog(), error, false);
                            ui.update_schemas_button.set_text(&qtr("updater_update_schemas_error"));
                        } else {
                            ui.update_schemas_button.set_text(&qtr("updater_update_schemas_updated"));
                        }
                    },
                    Response::Error(error) => {
                        show_dialog(ui.dialog(), error, false);
                        ui.update_schemas_button.set_text(&qtr("updater_update_schemas_error"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            }
        ));

        Self {
            update_program,
            update_schemas,
        }
    }
}
