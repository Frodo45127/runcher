launch_game = Launch Game
settings = Settings
open_folders = Open Folders...
open_game_root_folder = Open Game Root Folder
open_game_data_folder = Open Game Data Folder
open_game_content_folder = Open Game Content Folder
open_game_secondary_folder = Open Game Secondary Folder
open_game_config_folder = Open Game Config Folder
open_runcher_config_folder = Open Runcher Config Folder
open_runcher_error_folder = Open Runcher Error Folder
steam_api_key = Steam API Key

menu_bar_game_selected = Game Selected
menu_bar_about = About

about_qt = About QT
about_runcher = About Runcher
check_updates = Check Updates
check_schema_updates = Check Schema Updates

settings_game_line_ph = This is the folder where you have {"{"}{"}"} installed, where the .exe is.
default_game = Default Game
update_channel = Update Channel

pack_name = Pack Name
pack_path = Pack Path
location = Location
load_order = Load Order

load_profile = Load Profile
save_profile = Save Profile
profile_name = Profile Name

category_name = Category Name
category_new = New Category
category_new_placeholder = Category Name
category_delete = Delete Category
category_rename = Rename Category
categories_send_to_menu = Send to Category...
enable_selected = Enable Selected
disable_selected = Disable Selected

title_success = Success!
title_error = Error!

update_checker = Update Checker
update_searching = Searching for updates…
update_button = Update
api_response_success_new_stable_update = <h4>New major stable update found: {"{"}{"}"}</h4> <p>Please, make sure to save your work in progress before hitting 'Update', or you may lose it.</p>
api_response_success_new_beta_update = <h4>New beta update found: {"{"}{"}"}</h4><p>Please, make sure to save your work in progress before hitting 'Update', or you may lose it.</p>
api_response_success_new_update_hotfix = <h4>New minor update/hotfix found: {"{"}{"}"}</h4><p>Please, make sure to save your work in progress before hitting 'Update', or you may lose it.</p>
api_response_success_no_update = <h4>No new updates available</h4> <p>More luck next time :)</p>
api_response_success_unknown_version = <h4>Error while checking new updates</h4> <p>There has been a problem when getting the latest released version number, or the current version number. That means I fucked up the last release title. If you see this, please report it here:\n<a href=\"https://github.com/Frodo45127/runcher/issues\">https://github.com/Frodo45127/runcher/issues</a></p>
api_response_error = <h4>Error while checking new updates :(</h4> {"{"}{"}"}
update_in_prog = <p>Downloading updates, don't close this window…</p> <p>This may take a while.</p>
restart_button = Restart
update_success_main_program = <h4>Runcher updated correctly!</h4> <p>To check what changed in this update, check this link: <a href='file:///{"{"}{"}"}'>CHANGELOG.md</a>. If you're updating to a beta, the relevant changes are on the "Unreleased" section.</p> <p>Please, restart the program for the changes to apply.</p>

mod_name = Mod Name (Pack Name)
creator = Author
file_size = File Size
file_url = File Url
preview_url = Preview Url
time_created = Creation Date
time_updated = Last Update Date
last_check = Last Update Date of Mod Info

open_in_explorer = Open in Explorer
open_in_steam = Open in Steam Workshop

language = Language
dark_mode = Dark Mode
reload = Reload Mod List
check_updates_on_start = Check Updates on Start

copy_load_order = Copy Load Order
paste_load_order = Paste Load Order
load_order_string_info_paste = <p>Select the mode you want, and paste the String to import. Regarding the modes:</p>
    <ul>
        <li><b>Runcher mode:</b> select this if you're importing a Load Order String generated on another Runcher instance</li>
        <li><b>Modlist mode:</b> select this if you're importing a Load Order from a "used_mods.txt" or similar file. This is for importing load orders from other mod managers.</li>
    </ul>

load_order_string_info_copy = Copy this String, and paste it in another instance of Runcher to replicate this load order.
load_order_string_title_copy = Copy Load Order
load_order_string_title_paste = Paste Load Order

enable_logging = Enable Logging
enable_skip_intro = Skip Intro Videos
pack_type = Pack Type
steam_id = Steam ID
game_paths = Game Paths
flags = Flags

mod_flags_description = Possible problems detected:
mod_outdated_description = <li>
        <p><b>Outdated Mod</b>: This mod is for a previous version of the game and may need updating to work properly on the current version of the game.<p/>
        <p>Some examples of mods that usually do not need updates to keep working are:
            <ul>
                <li>Custom Maps</li>
            <ul/>
        <p/>
    </li>

unit_multiplier = Unit Multiplier

update_schema_checker = Update Schema Checker
update_no_local_schema = <p>No local schemas found. Do you want to download the latest ones?</p><p><b>NOTE:</b> Schemas are needed for certain advanced features.</p>
schema_no_update = <h4>No new schema updates available</h4> <p>More luck next time :)</p>
schema_new_update = <h4>New schema update available</h4> <p>Do you want to update the schemas?</p>
schema_update_success = <h4>Schemas updated and reloaded</h4><p>You can continue using Runcher now.</p>
check_schema_updates_on_start = Check Schema Updates on Start

steam_user_id = Steam User Id
merge_all_mods = Merge All Mods
enable_translations = Enable Translations

github_link = Open Runcher's Github Page
discord_link = Open "The Modding Den" discord channel
patreon_link = Support me on Patreon

translation_download_error = Error while trying to download the latest mod translations: {"{"}{"}"}. Using previously downloaded/local translations only instead.
date_format = Date Format

updater_title = Update Manager
updater_info_title = Info
updater_info = <p>This is the central update manager for Runcher. What each button means:</p>
    <ul>
        <li>
            <b>Program Updates</b>: Updates to the program itself. After updating it, you can click it again to restart into the updated program. Some notes on these updates:<ul>
                <li>To see the changes, after updating you can <a href='file:///{"{"}{"}"}'>click here</a> or you can open the CHANGELOG.md file in Runcher's folder.</li>
                <li>Note that there are two update channels: beta and stable. <b>You're currently on the {"{"}{"}"} channel</b>. You can change the channel in the settings.</li>
                <li>If you select "Stable" channel and you're in a beta, the latest Stable will be always shown as an available update, EVEN IF IT'S OLDER THAN YOUR BETA. This is to allow rollbacks. So if you want to use betas, make sure to select the "Beta" channel.</li>
            </ul>
        </li>
        <li><b>Schema Updates</b>: These files are required for some of the launch options. If you don't have them, some of the options may not work.</li>
    </ul>

updater_update_schemas = Schema Updates:
updater_update_program = Program Updates:
updater_update_schemas_checking = Checking, pls wait...
updater_update_program_checking = Checking, pls wait...

updater_update_program_available = Update {"{"}{"}"} available!
updater_update_program_no_updates = No updates found.

updater_update_schemas_available = Update available!
updater_update_schemas_no_updates = No update found.

updater_update_schemas_error = Error updating schemas.
updater_update_schemas_updated = Schemas updated!

updater_update_program_error = Error updating Runcher.
updater_update_program_updated = Runcher updated! Click here to restart it.

settings_font_title = Font Settings
edit_load_order_with_auto_on = Automatic load order mode is enabled. Disable it if you want to manually edit the load order.
game_config_error = Game config not loaded. Please configure the game correctly and try again.
automatic_mode_tooltip = Automatic Mode

    If this is enabled, the load order is automatically generated. If you want to manually edit it, disable this.

updater_update_program_updating = Updating, pls wait...
updater_update_schemas_updating = Updating, pls wait...

are_you_sure_title = Are you sure?
profile_manager = Profile Manager

profile_details_title = Profile Details
profile_rename = Rename Profile
profile_delete = Delete Profile
profile_shortcut_new = Create Shortcut
profile_manager_title = Profile Manager

are_you_sure_delete_profile = Are you sure you want to delete the currently selected profile?

profile_shortcut = New Shortcut
profile_shortcut_name = Shortcut Name
profile_shortcut_location = Shortcut Location
profile_shortcut_game = Game
profile_shortcut_autostart = Autostart Game?
select_location_folder = Select Link Location
profile_shortcut_icon = Shortcut Icon
select_icon = Select Shortcut Icon
category_sort = Sort Category
tools_title = Tools
tools_column_name = Tool Name
tools_column_path = Tool Path
tools_column_games = Games Supported
open_in_tool_menu = Open with Tool
tools_add = Add
tools_remove = Remove

data_list_title = Data Tree
pack_list_title = Pack List

game_data = Game Data
file_name = File/Folder Name
reload_data_view = Reload Data View

expand_all = Expand All
collapse_all = Collapse All
open_workshop_link_in_steam = Open Workshop Links on Steam

import_string_modlist_mode = Modlist Mode
import_string_runcher_mode = Runcher Mode

settings_secondary_mods_folder = Secondary Mods Folder
settings_secondary_mods_folder_ph = This is an alternative folder to place mods, so /data doesn't get too crowded.

upload_to_workshop_title = Upload/Update To Workshop
upload_to_workshop = Upload/Update To Workshop
upload_workshop_title = Title
upload_workshop_description = Description
upload_workshop_changelog = Changelog
upload_workshop_tag = Tag
upload_workshop_visibility = Visibility

upload_workshop_visibility_public = Public
upload_workshop_visibility_friends_only = Friends Only
upload_workshop_visibility_private = Private
upload_workshop_visibility_unlisted = Unlisted
