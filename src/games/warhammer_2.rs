//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::{anyhow, Result};


use std::path::PathBuf;

use rpfm_extensions::translator::{PackTranslation, TRANSLATED_PATH};

use rpfm_lib::files::{FileType, loc::Loc, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::GameInfo;

use crate::app_ui::AppUI;
use crate::games::EMPTY_CA_VP8;
use crate::settings_ui::translations_local_path;

const SCRIPT_DEBUG_ACTIVATOR_PATH: &str = "script/enable_console_logging";

const INTRO_MOVIE_PATHS_BY_GAME: [&str; 3] = [
    "movies/startup_movie_01.ca_vp8",
    "movies/startup_movie_02.ca_vp8",
    "movies/startup_movie_03.ca_vp8",
];

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub unsafe fn prepare_script_logging(reserved_pack: &mut Pack) -> Result<()> {
    let file = RFile::new_from_vec("why not working?!!".as_bytes(), FileType::Text, 0, SCRIPT_DEBUG_ACTIVATOR_PATH);
    reserved_pack.files_mut().insert(SCRIPT_DEBUG_ACTIVATOR_PATH.to_string(), file);

    Ok(())
}

pub unsafe fn prepare_skip_intro_videos(reserved_pack: &mut Pack) -> Result<()> {
    for path in INTRO_MOVIE_PATHS_BY_GAME {
        let file = RFile::new_from_vec(&EMPTY_CA_VP8, FileType::Video, 0, path);
        reserved_pack.files_mut().insert(path.to_string(), file);
    }

    Ok(())
}

pub unsafe fn prepare_translations(app_ui: &AppUI, game: &GameInfo, reserved_pack: &mut Pack) -> Result<()> {
    match translations_local_path() {
        Ok(path) => {
            let language = app_ui.actions_ui().enable_translations_combobox().current_text().to_std_string();
            let mut pack_names = (0..app_ui.pack_list_ui().model().row_count_0a())
                .filter_map(|index| PathBuf::from(app_ui.pack_list_ui().model().item_2a(index, 2).text().to_std_string()).file_name().map(|name| name.to_string_lossy().to_string()))
                .collect::<Vec<_>>();

            // Reversed so we just get the higher priority stuff at the end, overwriting the rest.
            pack_names.sort();
            pack_names.reverse();

            let mut loc = Loc::new();
            let mut loc_data = vec![];
            for pack_name in &pack_names {
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
                    }
                }
            }

            if !loc_data.is_empty() {
                loc.set_data(&loc_data)?;

                let file = RFile::new_from_decoded(&RFileDecoded::Loc(loc), 0, TRANSLATED_PATH);
                reserved_pack.files_mut().insert(TRANSLATED_PATH.to_string(), file);
            }

            Ok(())
        }
        Err(_) => Err(anyhow!("Failed to get local translations path. If you see this, it means I forgot to write the code to make sure that folder exists.")),
    }
}
