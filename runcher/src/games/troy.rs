//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;

use std::path::{Path, PathBuf};

use rpfm_lib::schema::Schema;
use rpfm_lib::files::{Container, ContainerPath, DecodeableExtraData, EncodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::GameInfo;

use crate::app_ui::AppUI;
//use crate::games::EMPTY_CA_VP8;

const SCRIPT_DEBUG_ACTIVATOR_PATH: &str = "script/enable_console_logging";

//const INTRO_MOVIE_PATHS_BY_GAME: [&str; 3] = [
//    "movies/startup_movie_01.ca_vp8",
//    "movies/startup_movie_02.ca_vp8",
//    "movies/startup_movie_03.ca_vp8",
//];

const INTRO_MOVIE_KEYS: [&str; 3] = [
    "startup_movie_01",
    "startup_movie_02",
    "startup_movie_03",
];

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub unsafe fn prepare_script_logging(reserved_pack: &mut Pack) -> Result<()> {
    let file = RFile::new_from_vec("why not working?!!".as_bytes(), FileType::Text, 0, SCRIPT_DEBUG_ACTIVATOR_PATH);
    reserved_pack.files_mut().insert(SCRIPT_DEBUG_ACTIVATOR_PATH.to_string(), file);

    Ok(())
}

pub unsafe fn prepare_skip_intro_videos(app_ui: &AppUI, game: &GameInfo, game_path: &Path, reserved_pack: &mut Pack, schema: &Schema) -> Result<()> {

    // Just replacing the files causes the game to crash, so we're going to remove them from the videos table.
    //for path in INTRO_MOVIE_PATHS_BY_GAME {
    //    let file = RFile::new_from_vec(&EMPTY_CA_VP8, FileType::Video, 0, path);
    //    reserved_pack.files_mut().insert(path.to_string(), file);
    //}

    let vanilla_pack = Pack::read_and_merge_ca_packs(game, game_path)?;
    let mut videos = vanilla_pack.files_by_path(&ContainerPath::Folder("db/videos_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let paths = (0..app_ui.pack_list_ui().model().row_count_0a())
        .map(|index| PathBuf::from(app_ui.pack_list_ui().model().item_2a(index, 2).text().to_std_string()))
        .collect::<Vec<_>>();

    if !paths.is_empty() {
        let modded_pack = Pack::read_and_merge(&paths, true, false)?;
        videos.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/videos_tables/".to_string()), true)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>());
    }

    videos.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/videos_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    // Decode each table, modify it, then re-encode it and add it.
    let enc_extra_data = Some(EncodeableExtraData::new_from_game_info(game));
    let mut dec_extra_data = DecodeableExtraData::default();
    dec_extra_data.set_schema(Some(schema));
    let dec_extra_data = Some(dec_extra_data);

    for table in &mut videos {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            for row in data.data_mut() {

                if let Some(DecodedData::StringU8(key)) = row.first().cloned() {

                    if INTRO_MOVIE_KEYS.contains(&&*key) {
                        if let Some(DecodedData::StringU8(value)) = row.get_mut(0) {
                            *value = "dummy".to_string();
                        }
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    Ok(())
}
