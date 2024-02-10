//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;

use rpfm_lib::files::{FileType, pack::Pack, RFile};

use crate::games::EMPTY_CA_VP8;

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
