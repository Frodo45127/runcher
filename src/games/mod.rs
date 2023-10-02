//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QString;

use anyhow::{anyhow, Result};

use std::path::{PathBuf, Path};

use rpfm_extensions::translator::*;

use rpfm_lib::files::{Container, FileType, loc::Loc, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::{*, supported_games::*};

use rpfm_ui_common::settings::*;

use crate::app_ui::AppUI;
use crate::SCHEMA;
use crate::translations_local_path;

const EMPTY_CA_VP8: [u8; 595] = [
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

const EMPTY_BIK: [u8; 520] = [
    0x42, 0x49, 0x4B, 0x69, 0x00, 0x02, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xC8, 0x01, 0x00, 0x00, 0x01, 0x00,
    0x00, 0x00, 0x80, 0x02, 0x00, 0x00, 0xE0, 0x01, 0x00, 0x00, 0x19, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x90, 0x1B, 0x00, 0x00, 0x44, 0xAC, 0x00, 0x70, 0x00, 0x00,
    0x00, 0x00, 0x41, 0x00, 0x00, 0x00, 0x08, 0x02, 0x00, 0x00, 0xE4, 0x00, 0x00, 0x00, 0x90, 0x1B, 0x00, 0x00,
    0x20, 0xF9, 0x1A, 0x30, 0xA5, 0xDB, 0xEF, 0xAF, 0x82, 0x12, 0x02, 0xB2, 0xC1, 0x91, 0xB1, 0x11, 0x42, 0x12,
    0xD2, 0x51, 0x61, 0x21, 0xF1, 0xE0, 0xC0, 0xE0, 0xC0, 0xC0, 0xD0, 0x40, 0x61, 0xE2, 0x85, 0x00, 0x82, 0x40,
    0x43, 0x16, 0x73, 0xD2, 0x29, 0x1A, 0x52, 0x68, 0x82, 0xA5, 0x85, 0x44, 0x8C, 0xE9, 0x0C, 0x71, 0x90, 0x82,
    0x84, 0x11, 0x25, 0x91, 0x13, 0x42, 0x05, 0x81, 0x4B, 0x34, 0x5B, 0x2C, 0x63, 0x15, 0x08, 0x89, 0x11, 0x02,
    0x09, 0xC6, 0x50, 0x94, 0x64, 0xE9, 0xA3, 0x02, 0x80, 0xE8, 0x44, 0xA3, 0x88, 0x5F, 0x01, 0x28, 0x40, 0x4A,
    0x68, 0x54, 0x40, 0x2D, 0x80, 0xC0, 0x1C, 0x01, 0x09, 0x84, 0x00, 0x41, 0x24, 0x00, 0xFB, 0xC3, 0x87, 0x0F,
    0x1F, 0x10, 0x84, 0x23, 0x94, 0x2A, 0x24, 0x05, 0x4B, 0x21, 0x01, 0xE9, 0xD0, 0xD0, 0xB0, 0xC8, 0xF8, 0xE0,
    0xD8, 0xB0, 0x90, 0x88, 0x50, 0x40, 0x48, 0x38, 0x38, 0x40, 0x50, 0x68, 0xA0, 0x30, 0xF1, 0x3A, 0x04, 0x84,
    0x04, 0x85, 0x66, 0x79, 0x2F, 0xDC, 0x28, 0x2D, 0x15, 0x51, 0x96, 0xCB, 0x43, 0xAC, 0xD8, 0x0B, 0x54, 0x46,
    0x08, 0xC2, 0x25, 0xA2, 0xCA, 0x31, 0x04, 0xD6, 0x69, 0x13, 0x94, 0xCB, 0xCC, 0x12, 0x84, 0x34, 0xC1, 0x4D,
    0xAE, 0x3A, 0xEA, 0x50, 0x8B, 0x28, 0x0C, 0xC7, 0x35, 0x10, 0x8D, 0xD2, 0x4C, 0x1C, 0x88, 0x12, 0x30, 0x04,
    0x48, 0x59, 0x04, 0x09, 0x5B, 0x1C, 0x24, 0x00, 0x00, 0x10, 0x00, 0x60, 0x03, 0x30, 0x00, 0x80, 0x70, 0x00,
    0xF8, 0xF0, 0xE1, 0xC3, 0x07, 0x00, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x57, 0xC1, 0x7F, 0x65, 0xFC, 0x10,
    0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x80, 0x20, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB,
    0xB6, 0x01, 0x82, 0xB4, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB, 0x06, 0x08, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x40, 0x90, 0xB6, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB, 0x00, 0x41, 0xDA, 0xB6, 0x6D, 0xDB, 0xB6, 0x6D,
    0x03, 0x04, 0x11, 0x04, 0x69, 0xDB, 0xB6, 0x6D, 0xDB, 0xB6, 0x0D, 0x10, 0xA4, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB,
    0x36, 0x00, 0x85, 0xB6, 0xFD, 0xFF, 0x00, 0x14, 0x04, 0x28, 0xDA, 0xB6, 0x6D, 0xFB, 0xFF, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x57, 0xC1, 0x7E, 0x65, 0xEC, 0x00, 0x00, 0x00, 0x00, 0x11, 0x08, 0x00, 0x00, 0x00,
    0x11, 0x00, 0x00, 0x80, 0x20, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
    0x4B, 0xFC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F, 0x58, 0x22, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0xA0, 0xE0, 0xFF, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x57, 0xC1, 0x7E, 0x65, 0xEC, 0x00, 0x00, 0x00,
    0x00, 0x11, 0x08, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x80, 0x20, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x4B, 0xFC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F, 0x58, 0x22, 0x02,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA0, 0xE0, 0xFF, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00,
];

mod attila;
mod empire;
mod napoleon;
mod pharaoh;
mod rome_2;
mod shogun_2;
mod three_kingdoms;
mod thrones;
mod troy;
mod warhammer;
mod warhammer_2;
mod warhammer_3;

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub unsafe fn setup_launch_options(app_ui: &AppUI, game: &GameInfo, game_path: &Path) {

    // The blockers are needed to avoid issues with game change causing incorrect status to be saved.
    app_ui.actions_ui().enable_logging_checkbox().block_signals(true);
    app_ui.actions_ui().enable_skip_intro_checkbox().block_signals(true);
    app_ui.actions_ui().enable_translations_combobox().block_signals(true);
    app_ui.actions_ui().merge_all_mods_checkbox().block_signals(true);
    app_ui.actions_ui().unit_multiplier_spinbox().block_signals(true);
    app_ui.actions_ui().open_game_content_folder().block_signals(true);
    app_ui.actions_ui().save_combobox().block_signals(true);

    // Only set enabled the launch options that work for the current game.
    match game.key() {
        KEY_PHARAOH => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_WARHAMMER_3 => {
            let schema = SCHEMA.read().unwrap();
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(schema.is_some());
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_TROY => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_THREE_KINGDOMS => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);    // 3K doesn't support logging by default.
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_WARHAMMER_2 => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_WARHAMMER => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);    // Warhammer 1 doesn't support logging by default.
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_THRONES_OF_BRITANNIA => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_ATTILA => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_ROME_2 => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(true);
        },
        KEY_SHOGUN_2 => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(true);
            app_ui.actions_ui().save_combobox().set_enabled(false);
        },
        KEY_NAPOLEON => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(false);
            app_ui.actions_ui().save_combobox().set_enabled(false);
        },
        KEY_EMPIRE => {
            app_ui.actions_ui().enable_logging_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().enable_skip_intro_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().enable_translations_combobox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(true);
            app_ui.actions_ui().unit_multiplier_spinbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);
            app_ui.actions_ui().open_game_content_folder().set_enabled(false);
            app_ui.actions_ui().save_combobox().set_enabled(false);
        }
        &_ => {},
    }

    // Disable this until I figure out how to fix the performance problems, and I change the pack to be on /data
    app_ui.actions_ui().merge_all_mods_checkbox().parent().static_downcast::<qt_widgets::QWidget>().set_enabled(false);

    // Update the launch options for the new game.
    app_ui.actions_ui().enable_logging_checkbox().set_checked(setting_bool(&format!("enable_logging_{}", game.key())));
    app_ui.actions_ui().enable_skip_intro_checkbox().set_checked(setting_bool(&format!("enable_skip_intros_{}", game.key())));
    app_ui.actions_ui().merge_all_mods_checkbox().set_checked(setting_bool(&format!("merge_all_mods_{}", game.key())));
    app_ui.actions_ui().unit_multiplier_spinbox().set_value({
        let value = setting_f32(&format!("unit_multiplier_{}", game.key()));
        if value == 0.00 {
            1.00
        } else {
            value
        }
    } as f64);

    // Populate the list of translations depending on what local_XX packs the game has.
    app_ui.actions_ui().enable_translations_combobox().clear();
    app_ui.actions_ui().enable_translations_combobox().insert_item_int_q_string(0, &QString::from_std_str("--"));
    app_ui.actions_ui().enable_translations_combobox().set_current_index(0);

    if let Ok(ca_packs) = game.ca_packs_paths(game_path) {
        let mut languages = ca_packs.iter()
            .filter_map(|path| path.file_stem())
            .filter(|name| name.to_string_lossy().starts_with("local_"))
            .map(|name| name.to_string_lossy().split_at(6).1.to_uppercase())
            .collect::<Vec<_>>();

        // Sort, and remove anything longer than 2 characters to avoid duplicates.
        languages.retain(|lang| lang.chars().count() == 2);
        languages.sort();

        for (index, language) in languages.iter().enumerate() {
            app_ui.actions_ui().enable_translations_combobox().insert_item_int_q_string(index as i32 + 1, &QString::from_std_str(language));
        }

        let language_to_select = setting_string(&format!("enable_translations_{}", game.key()));
        app_ui.actions_ui().enable_translations_combobox().set_current_text(&QString::from_std_str(language_to_select));
    }

    // Unblock all blocked signals.
    app_ui.actions_ui().enable_logging_checkbox().block_signals(false);
    app_ui.actions_ui().enable_skip_intro_checkbox().block_signals(false);
    app_ui.actions_ui().enable_translations_combobox().block_signals(false);
    app_ui.actions_ui().merge_all_mods_checkbox().block_signals(false);
    app_ui.actions_ui().unit_multiplier_spinbox().block_signals(false);
    app_ui.actions_ui().save_combobox().block_signals(false);
    app_ui.actions_ui().open_game_content_folder().block_signals(false);
}

pub unsafe fn prepare_unit_multiplier(app_ui: &AppUI, game: &GameInfo, game_path: &Path, reserved_pack: &mut Pack) -> Result<()> {
    match *SCHEMA.read().unwrap() {
        Some(ref schema) => {
            if app_ui.actions_ui().unit_multiplier_spinbox().is_enabled() && app_ui.actions_ui().unit_multiplier_spinbox().value() != 1.00 {
                match game.key() {
                    KEY_PHARAOH => Ok(()),
                    KEY_WARHAMMER_3 => warhammer_3::prepare_unit_multiplier(app_ui, game, game_path, reserved_pack, schema),
                    KEY_TROY |
                    KEY_THREE_KINGDOMS |
                    KEY_WARHAMMER_2 |
                    KEY_WARHAMMER |
                    KEY_THRONES_OF_BRITANNIA |
                    KEY_ATTILA |
                    KEY_ROME_2 |
                    KEY_SHOGUN_2 |
                    KEY_NAPOLEON |
                    KEY_EMPIRE => Ok(()),
                    _ => Ok(())
                }
            } else {
                Ok(())
            }
        }
        None => Ok(())
    }
}

pub unsafe fn prepare_script_logging(app_ui: &AppUI, game: &GameInfo, reserved_pack: &mut Pack) -> Result<()> {
    if app_ui.actions_ui().enable_logging_checkbox().is_enabled() && app_ui.actions_ui().enable_logging_checkbox().is_checked() {
        match game.key() {
            KEY_PHARAOH => pharaoh::prepare_script_logging(reserved_pack),
            KEY_WARHAMMER_3 => warhammer_3::prepare_script_logging(reserved_pack),
            KEY_TROY => troy::prepare_script_logging(reserved_pack),
            KEY_THREE_KINGDOMS => Ok(()),
            KEY_WARHAMMER_2 => warhammer_2::prepare_script_logging(reserved_pack),
            KEY_WARHAMMER |
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 |
            KEY_SHOGUN_2 |
            KEY_NAPOLEON |
            KEY_EMPIRE => Ok(()),
            _ => Ok(())
        }
    } else {
        Ok(())
    }
}

pub unsafe fn prepare_skip_intro_videos(app_ui: &AppUI, game: &GameInfo, game_path: &Path, reserved_pack: &mut Pack) -> Result<()> {
    if app_ui.actions_ui().enable_skip_intro_checkbox().is_enabled() && app_ui.actions_ui().enable_skip_intro_checkbox().is_checked() {
        match game.key() {
            KEY_PHARAOH => match *SCHEMA.read().unwrap() {
                Some(ref schema) => pharaoh::prepare_skip_intro_videos(app_ui, game, game_path, reserved_pack, schema),
                None => Ok(())
            }
            KEY_WARHAMMER_3 => warhammer_3::prepare_skip_intro_videos(reserved_pack),
            KEY_TROY => match *SCHEMA.read().unwrap() {
                Some(ref schema) => troy::prepare_skip_intro_videos(app_ui, game, game_path, reserved_pack, schema),
                None => Ok(())
            }
            KEY_THREE_KINGDOMS => three_kingdoms::prepare_skip_intro_videos(reserved_pack),
            KEY_WARHAMMER_2 => warhammer_2::prepare_skip_intro_videos(reserved_pack),
            KEY_WARHAMMER => warhammer::prepare_skip_intro_videos(reserved_pack),
            KEY_THRONES_OF_BRITANNIA => thrones::prepare_skip_intro_videos(reserved_pack),
            KEY_ATTILA => attila::prepare_skip_intro_videos(reserved_pack),
            KEY_ROME_2 => rome_2::prepare_skip_intro_videos(reserved_pack),
            KEY_SHOGUN_2 => shogun_2::prepare_skip_intro_videos(reserved_pack),
            KEY_NAPOLEON => napoleon::prepare_skip_intro_videos(reserved_pack),
            KEY_EMPIRE => empire::prepare_skip_intro_videos(reserved_pack),
            _ => Ok(())
        }
    } else {
        Ok(())
    }
}

/// All total war games use the same translation system.
///
/// The only particularity is that all games before warhammer 1 need to merge all translations into a localisation.loc file.
pub unsafe fn prepare_translations(app_ui: &AppUI, game: &GameInfo, reserved_pack: &mut Pack) -> Result<()> {

    // TODO: Troy has a weird translation system. Check that it works, and check pharaoh too.
    if app_ui.actions_ui().enable_translations_combobox().is_enabled() && app_ui.actions_ui().enable_translations_combobox().current_index() != 0 {
        match translations_local_path() {
            Ok(path) => {
                let language = app_ui.actions_ui().enable_translations_combobox().current_text().to_std_string();
                let mut pack_paths = (0..app_ui.pack_list_ui().model().row_count_0a())
                    .map(|index| PathBuf::from(app_ui.pack_list_ui().model().item_2a(index, 2).text().to_std_string()))
                    .collect::<Vec<_>>();

                // Reversed so we just get the higher priority stuff at the end, overwriting the rest.
                pack_paths.sort();
                pack_paths.reverse();

                // If we need to merge the localisation.loc file if found to the translations.
                let use_old_multilanguage_logic = matches!(game.key(),
                    KEY_THRONES_OF_BRITANNIA |
                    KEY_ATTILA |
                    KEY_ROME_2 |
                    KEY_SHOGUN_2 |
                    KEY_NAPOLEON |
                    KEY_EMPIRE
                );

                let mut loc = Loc::new();
                let mut loc_data = vec![];
                for pack_path in &pack_paths {
                    if let Some(ref pack_name) = pack_path.file_name().map(|name| name.to_string_lossy().to_string()) {
                        if let Ok(tr) = PackTranslation::load(&path, pack_name, game.key(), &language) {
                            for tr in tr.translations().values() {

                                // Only add entries for values we actually have translated and up to date.
                                if !tr.value_translated().is_empty() && !*tr.needs_retranslation() {
                                    loc_data.push(vec![
                                        DecodedData::StringU16(tr.key().to_owned()),
                                        DecodedData::StringU16(tr.value_translated().to_owned()),
                                        DecodedData::Boolean(false),
                                    ]);
                                }

                                // If we're in a game with the old logic and there is no translation, add the text in english directly.
                                else if use_old_multilanguage_logic && !tr.value_original().is_empty() {
                                    loc_data.push(vec![
                                        DecodedData::StringU16(tr.key().to_owned()),
                                        DecodedData::StringU16(tr.value_original().to_owned()),
                                        DecodedData::Boolean(false),
                                    ]);
                                }
                            }
                        }

                        // If there's no translation data, just merge their locs.
                        else {
                            let mut pack = Pack::read_and_merge(&[pack_path.to_path_buf()], true, false)?;

                            let mut locs = pack.files_by_type_mut(&[FileType::Loc]);
                            locs.sort_by(|a, b| a.path_in_container_raw().cmp(b.path_in_container_raw()));

                            let locs_split = locs.iter_mut()
                                .filter_map(|loc| if let Ok(Some(RFileDecoded::Loc(loc))) = loc.decode(&None, false, true) {
                                    Some(loc)
                                } else {
                                    None
                                })
                                .collect::<Vec<_>>();

                            let locs_split_ref = locs_split.iter().collect::<Vec<_>>();

                            let mut merged_loc = Loc::merge(&locs_split_ref)?;
                            loc_data.append(merged_loc.data_mut());
                        }
                    }
                }

                // If the game uses the old multilanguage logic, we need to get the most updated version of localisation.loc from the game and append it to our loc.
                if use_old_multilanguage_logic {
                    let game_path = setting_path(game.key());
                    let mut pack = Pack::read_and_merge_ca_packs(game, &game_path)?;
                    if let Some(vanilla_loc) = pack.file_mut(TRANSLATED_PATH_OLD, false) {
                        if let Ok(Some(RFileDecoded::Loc(mut loc))) = vanilla_loc.decode(&None, false, true) {
                            loc_data.append(loc.data_mut());
                        }
                    }
                }

                if !loc_data.is_empty() {
                    loc.set_data(&loc_data)?;

                    let path = if use_old_multilanguage_logic {
                        TRANSLATED_PATH_OLD.to_string()
                    } else {
                        TRANSLATED_PATH.to_string()
                    };

                    let file = RFile::new_from_decoded(&RFileDecoded::Loc(loc), 0, &path);
                    reserved_pack.files_mut().insert(path, file);
                }

                Ok(())
            }
            Err(_) => Err(anyhow!("Failed to get local translations path. If you see this, it means I forgot to write the code to make sure that folder exists.")),
        }
    } else {
        Ok(())
    }
}

