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
use regex::Regex;
use steam_workshop_api::client::Workshop;
use steam_workshop_api::interfaces::{i_steam_remote_storage::*, i_steam_user::*};

use std::collections::HashMap;

use rpfm_ui_common::settings::setting_string;

use crate::integrations::Mod;

lazy_static::lazy_static! {
    pub static ref REGEX_URL: Regex = Regex::new(r"(\[url=)(.*)(\])(.*)(\[/url\])").unwrap();
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//


//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//
/*
pub fn info_for_mod(mod_id: &str) -> Result<Vec<WorkshopItem>> {
    let client = Workshop::new(None);
    get_published_file_details(&client, &[mod_id.to_string()]).map_err(From::from)
}

pub fn info_for_mods(mod_ids: &[String]) -> Result<Vec<WorkshopItem>> {
    let client = Workshop::new(None);
    get_published_file_details(&client, mod_ids).map_err(From::from)
}*/

pub fn populate_mods(mods: &mut HashMap<String, Mod>, mod_ids: &[String], last_update_date: u64) -> Result<()> {
    let client = Workshop::new(None);
    let workshop_items = get_published_file_details(&client, mod_ids)?;
    for workshop_item in workshop_items {
        if *workshop_item.result() == 1 {
            if let Some(modd) = mods.values_mut().filter(|modd| modd.steam_id().is_some()).find(|modd| &modd.steam_id().clone().unwrap() == workshop_item.publishedfileid()) {
                modd.set_name(workshop_item.title().clone().unwrap());
                modd.set_creator(workshop_item.creator().clone().unwrap());
                modd.set_file_size(workshop_item.file_size().unwrap());
                modd.set_file_url(workshop_item.file_url().clone().unwrap());
                modd.set_preview_url(workshop_item.preview_url().clone().unwrap());
                modd.set_description(workshop_item.description().clone().unwrap());
                modd.set_time_created(workshop_item.time_created().unwrap());
                modd.set_time_updated(workshop_item.time_updated().unwrap());

                modd.set_outdated(last_update_date > *modd.time_updated() as u64);
            }
        }
    }

    let user_ids = mods.values()
        .filter_map(|modd| if !modd.creator().is_empty() {
            Some(modd.creator().to_owned())
        } else { None }
        ).collect::<Vec<_>>();

    populate_user_names(mods, &user_ids)?;

    Ok(())
}

pub fn populate_user_names(mods: &mut HashMap<String, Mod>, user_ids: &[String]) -> Result<()> {
    let mut client = Workshop::new(None);
    let api_key = setting_string("steam_api_key");
    if !api_key.is_empty() {
        client.set_apikey(Some(api_key));
        let user_names = get_player_names(&client, user_ids)?;

        for modd in mods.values_mut() {
            if let Some(creator_name) = user_names.get(modd.creator()) {
                modd.set_creator_name(creator_name.to_string());
            }
        }
    }

    Ok(())
}

/*
pub fn parse_to_html(string: &str) -> String {
    let mut new_string = "<html>".to_owned();

    new_string.push_str(&string.replace("[h1]", "<h1>").replace("[/h1]", "</h1>")
        .replace("[b]", "<b>").replace("[/b]", "</b>").replace("[B]", "<b>").replace("[/B]", "</b>")
        .replace("[i]", "<i>").replace("[/i]", "</i>")
        //.replace("[strike]", "<i>").replace("[/strike]", "</i>")
        //.replace("[spoiler]", "<i>").replace("[/spoiler]", "</i>")
        //.replace("[noparse]", "<i>").replace("[/noparse]", "</i>")
        .replace("[hr]", "<hr>").replace("[/hr]", "</hr>")
        .replace("[img]", "<img src=\"").replace("[/img]", "\"/>")
        // Missing url/img parsers here.
        .replace("[list]", "<ul>").replace("[/list]", "</ul>")
        .replace("[olist]", "<ol>").replace("[/olist]", "</ol>")
        .replace("[*]", "</li><li>")
        .replace("[quote]", "<blockquote>").replace("[/quote]", "</blockquote>")
        .replace("[code]", "<code>").replace("[/code]", "</code>")
        .replace("[table]", "<table>").replace("[/table]", "</table>")
        .replace("[th]", "<th>").replace("[/th]", "</th>")
        .replace("[tr]", "<tr>").replace("[/tr]", "</tr>")
        .replace("[td]", "<td>").replace("[/td]", "</td>")

        // Line jumps.
        .replace("\r\n", "<br/>")
        .replace('\n', "<br/>"));
    new_string.push_str("</html>");

    // Replace urls before the rest, as they require regexes.
    new_string = REGEX_URL.replace_all(&new_string, r"<url src='\2>\4</url>").to_string();
    new_string
}*/
