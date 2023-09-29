//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/runcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/runcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;

use rpfm_lib::games::pfh_file_type::PFHFileType;

use super::game_config::GameConfig;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct LoadOrder {

    // Id/Pack name of the mod. To get more data of the mod use this as key on the GameConfig/Mods hashmap.
    mods: Vec<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl LoadOrder {
    pub fn generate(&mut self, game_config: &GameConfig) {
        self.mods.clear();

        self.filter_and_sort(game_config);
    }

    pub fn filter_and_sort(&mut self, game_config: &GameConfig) {

        // Pre-sort the mods, with movie mods at the end.
        self.mods = game_config.mods()
            .values()
            .filter(|modd| (*modd.enabled() || *modd.pack_type() == PFHFileType::Movie) && !modd.paths().is_empty())
            .map(|modd| modd.id().to_string())
            .collect::<Vec<_>>();

        self.mods.sort_by(|a, b| {
            let a = game_config.mods().get(a).unwrap();
            let b = game_config.mods().get(b).unwrap();

            match a.pack_type().cmp(b.pack_type()) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => a.id().cmp(b.id()),
                Ordering::Less => Ordering::Less,
            }
        });

        // TODO: Automatically put parent mods above their children.
        // TODO2: If it works how I think it works, the game loads parent mods twice:
        // - First, when the're loaded as a mod.
        // - Second, when a child mod tries to load.
        //
        // That's what I could find from checking save mods. Need to check if that's true and if it's,
        // remove the parent mod from the final load order so it only loads once.
    }
}
