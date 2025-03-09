//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with Updates and the Main Program Update Checker.
!*/

use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QDialog;
use qt_widgets::{QWidget, QPushButton, QDialogButtonBox, QLabel, QGroupBox};

use qt_core::QBox;
use qt_core::QPtr;

use anyhow::Result;
use getset::*;
use rpfm_lib::integrations::git::GitResponse;

use std::rc::Rc;

use common_utils::updater::*;

use rpfm_ui_common::locale::{qtr, qtre};
use rpfm_ui_common::PROGRAM_PATH;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::updater_ui::slots::UpdaterUISlots;

const CHANGELOG_FILE: &str = "CHANGELOG.txt";

pub const STABLE: &str = "Stable";
pub const BETA: &str = "Beta";

const VIEW_DEBUG: &str = "ui_templates/updater_dialog.ui";
const VIEW_RELEASE: &str = "ui/updater_dialog.ui";

mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct UpdaterUI {
    main_widget: QBox<QWidget>,
    update_schemas_button: QPtr<QPushButton>,
    update_program_button: QPtr<QPushButton>,
    update_sql_scripts_button: QPtr<QPushButton>,
    accept_button: QPtr<QPushButton>,
    cancel_button: QPtr<QPushButton>,
}

//---------------------------------------------------------------------------//
//                              UI functions
//---------------------------------------------------------------------------//

impl UpdaterUI {

    /// This function checks for updates, and if it find any update, it shows the update dialog.
    pub unsafe fn new_with_precheck(app_ui: &Rc<AppUI>) -> Result<()> {
        let mut update_available = false;

        let updates_for_program = if setting_bool("check_updates_on_start") {
            let channel = TryFrom::try_from(&*setting_string("update_channel"))?;
            let receiver = CENTRAL_COMMAND.send_network(Command::CheckUpdates(channel));
            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::APIResponse(response) => {
                    match response {
                        APIResponse::NewStableUpdate(_) |
                        APIResponse::NewBetaUpdate(_) |
                        APIResponse::NewUpdateHotfix(_) => {
                            update_available |= true;
                        }
                        _ => {},
                    }
                    Some(response)
                }

                Response::Error(_) => None,
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        } else {
            None
        };

        let updates_for_schema = if setting_bool("check_schema_updates_on_start") {
            let receiver = CENTRAL_COMMAND.send_network(Command::CheckSchemaUpdates);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::APIResponseGit(response) => {
                    match response {
                        GitResponse::NoLocalFiles |
                        GitResponse::NewUpdate |
                        GitResponse::Diverged => {
                            update_available |= true;
                        }
                        _ => {},
                    }
                    Some(response)
                }

                Response::Error(_) => None,
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        } else {
            None
        };

        let updates_for_sql_scripts = if setting_bool("check_sql_scripts_updates_on_start") {
            let receiver = CENTRAL_COMMAND.send_network(Command::CheckSqlScriptsUpdates);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::APIResponseGit(response) => {
                    match response {
                        GitResponse::NoLocalFiles |
                        GitResponse::NewUpdate |
                        GitResponse::Diverged => {
                            update_available |= true;
                        }
                        _ => {},
                    }
                    Some(response)
                }

                Response::Error(_) => None,
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        } else {
            None
        };

        // Only show the dialog if there are updates.
        if update_available {
            Self::new(app_ui, updates_for_program, updates_for_schema, updates_for_sql_scripts)?;
        }

        Ok(())
    }

    pub unsafe fn new(app_ui: &Rc<AppUI>, precheck_program: Option<APIResponse>, precheck_schema: Option<GitResponse>, precheck_sql_scripts: Option<GitResponse>) -> Result<()> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;

        let info_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "info_groupbox")?;
        let info_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "info_label")?;
        let update_schemas_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "update_schemas_label")?;
        let update_program_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "update_program_label")?;
        let update_sql_scripts_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "update_sql_scripts_label")?;
        let update_schemas_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "update_schemas_button")?;
        let update_program_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "update_program_button")?;
        let update_sql_scripts_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "update_sql_scripts_button")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        let accept_button: QPtr<QPushButton> = button_box.button(StandardButton::Ok);
        let cancel_button: QPtr<QPushButton> = button_box.button(StandardButton::Cancel);

        let changelog_path = PROGRAM_PATH.join(CHANGELOG_FILE);

        info_groupbox.set_title(&qtr("updater_info_title"));
        info_label.set_text(&qtre("updater_info", &[&changelog_path.to_string_lossy(), &setting_string("update_channel")]));
        info_label.set_open_external_links(true);

        update_program_label.set_text(&qtr("updater_update_program"));
        update_schemas_label.set_text(&qtr("updater_update_schemas"));
        update_sql_scripts_label.set_text(&qtr("updater_update_sql_scripts"));

        update_program_button.set_text(&qtr("updater_update_program_checking"));
        update_schemas_button.set_text(&qtr("updater_update_schemas_checking"));
        update_sql_scripts_button.set_text(&qtr("updater_update_sql_scripts_checking"));

        update_program_button.set_enabled(false);
        update_schemas_button.set_enabled(false);
        update_sql_scripts_button.set_enabled(false);

        // Show the dialog before checking for updates.
        main_widget.static_downcast::<QDialog>().set_window_title(&qtr("updater_title"));
        main_widget.static_downcast::<QDialog>().show();

        // If we have prechecks done, do not re-check for updates on them.
        match precheck_program {
            Some(response) => {
                match response {
                    APIResponse::NewStableUpdate(last_release) |
                    APIResponse::NewBetaUpdate(last_release) |
                    APIResponse::NewUpdateHotfix(last_release) => {
                        update_program_button.set_text(&qtre("updater_update_program_available", &[&last_release]));
                        update_program_button.set_enabled(true);
                    }
                    APIResponse::NoUpdate |
                    APIResponse::UnknownVersion => {
                        update_program_button.set_text(&qtr("updater_update_program_no_updates"));
                    }
                }
            }
            None => {
                let channel = TryFrom::try_from(&*setting_string("update_channel"))?;
                let receiver = CENTRAL_COMMAND.send_network(Command::CheckUpdates(channel));
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::APIResponse(response) => {
                        match response {
                            APIResponse::NewStableUpdate(last_release) |
                            APIResponse::NewBetaUpdate(last_release) |
                            APIResponse::NewUpdateHotfix(last_release) => {
                                update_program_button.set_text(&qtre("updater_update_program_available", &[&last_release]));
                                update_program_button.set_enabled(true);
                            }
                            APIResponse::NoUpdate |
                            APIResponse::UnknownVersion => {
                                update_program_button.set_text(&qtr("updater_update_program_no_updates"));
                            }
                        }
                    }

                    Response::Error(_) => {
                        update_program_button.set_text(&qtr("updater_update_program_no_updates"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            },
        }

        match precheck_schema {
            Some(response) => {
                match response {
                    GitResponse::NoLocalFiles |
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_schemas_button.set_text(&qtr("updater_update_schemas_available"));
                        update_schemas_button.set_enabled(true);
                    }
                    GitResponse::NoUpdate => {
                        update_schemas_button.set_text(&qtr("updater_update_schemas_no_updates"));
                    }
                }
            }
            None => {
                let receiver = CENTRAL_COMMAND.send_network(Command::CheckSchemaUpdates);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::APIResponseGit(response) => {
                        match response {
                            GitResponse::NoLocalFiles |
                            GitResponse::NewUpdate |
                            GitResponse::Diverged => {
                                update_schemas_button.set_text(&qtr("updater_update_schemas_available"));
                                update_schemas_button.set_enabled(true);
                            }
                            GitResponse::NoUpdate => {
                                update_schemas_button.set_text(&qtr("updater_update_schemas_no_updates"));
                            }
                        }
                    }

                    Response::Error(_) => {
                        update_schemas_button.set_text(&qtr("updater_update_schemas_no_updates"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            },
        }

        match precheck_sql_scripts {
            Some(response) => {
                match response {
                    GitResponse::NoLocalFiles |
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_sql_scripts_button.set_text(&qtr("updater_update_sql_scripts_available"));
                        update_sql_scripts_button.set_enabled(true);
                    }
                    GitResponse::NoUpdate => {
                        update_sql_scripts_button.set_text(&qtr("updater_update_sql_scripts_no_updates"));
                    }
                }
            }
            None => {
                let receiver = CENTRAL_COMMAND.send_network(Command::CheckSchemaUpdates);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::APIResponseGit(response) => {
                        match response {
                            GitResponse::NoLocalFiles |
                            GitResponse::NewUpdate |
                            GitResponse::Diverged => {
                                update_sql_scripts_button.set_text(&qtr("updater_update_sql_scripts_available"));
                                update_sql_scripts_button.set_enabled(true);
                            }
                            GitResponse::NoUpdate => {
                                update_sql_scripts_button.set_text(&qtr("updater_update_sql_scripts_no_updates"));
                            }
                        }
                    }

                    Response::Error(_) => {
                        update_sql_scripts_button.set_text(&qtr("updater_update_sql_scripts_no_updates"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            },
        }

        let ui = Rc::new(Self {
            main_widget,
            update_schemas_button,
            update_program_button,
            update_sql_scripts_button,
            accept_button,
            cancel_button,
        });

        let slots = UpdaterUISlots::new(&ui, app_ui);
        ui.set_connections(&slots);

        Ok(())
    }

    pub unsafe fn set_connections(&self, slots: &UpdaterUISlots) {
        self.update_program_button.released().connect(slots.update_program());
        self.update_schemas_button.released().connect(slots.update_schemas());
        self.update_sql_scripts_button.released().connect(slots.update_sql_scripts());

        self.accept_button.released().connect(self.dialog().slot_accept());
        self.cancel_button.released().connect(self.dialog().slot_close());
    }

    pub unsafe fn dialog(&self) -> QPtr<QDialog> {
        self.main_widget().static_downcast::<QDialog>()
    }
}
