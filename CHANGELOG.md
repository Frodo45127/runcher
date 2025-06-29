# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

If you're looking for the changes included in the latest beta (against the latest stable version), check the unreleased section.

## [0.9.16]
### Fixed
- Fixed remaining issues (hopefully) regarding interactions between Runcher and TWPatcher/Workshopper.
- Fixed SQL vanilla db regenerating more often than it should.

## [0.9.15]
### Fixed
- Fixed TWPatcher not saving the patched pack correctly.
- Fixed ominous message when TWPatcher fails to generate a patch.

## [0.9.14]
### Changed
- Updated RPFM lib to 4.4.2, to support Lz4 and Zstd compressed files.
- TWPatcher output error should now be shown when it fails to execute.

## [0.9.13]
### Fixed
- Fixed "file not found" and other errors caused by windows using the wrong terminal to launch programs.

## [0.9.12]
### Changed
- SQL script processing should be a lot faster now.
- New icon for Thrones, courtesy of [@Mr.Tarzan](https://github.com/new-denim).

### Removed
- Removed Windows 7 support.

### Fixed
- Fixed wrong overwriting order on data view.
- Fixed wrong overwriting order on patching process.
- Fixed mod list in profiles ui not displaying the correct order.

## [0.9.11]
### Changed
- Movie packs can only be disabled in games since Warhammer 1. Older games can only disable them if they're not in /data.

### Fixed
- Fixed multiple path checks that caused movie packs to not be detected correctly as being in /data.
- Fixed a double add_working_directory command being added to the mod list when using mods in the secondary folder.

## [0.9.10]
### Added
- Implemented "Remove Siege Attacker" launch option.
- Implemented "Enable Dev-Only UI" launch option.
- Implemented experimental support for applying SQL scripts as patches to the load order.
- Implemented support for SQL scripts distributed inside Packs.
- Implemented support for presets for configurable SQL scripts distributed inside Packs.

### Changed
- Migrated patcher logic to a separate redistributeable exe: [TWPatcher](https://github.com/Frodo45127/twpatcher).
- Migrated workshop logic to a separate repo: [Workshopper](https://github.com/Frodo45127/workshopper)
- Improved compilation support.
- Disabled log-checking logic by default.
- Movie packs can now be disabled, even when they're in data.
- Runcher no longer checks if you're the author before uploading/updating a mod in the Workshop. This should allow (if the rumors are true and Steam actually allows it) contributors to push updates in released mods. 
- The TWPatcher terminal is now visible, so you know Runcher is doing something for the few seconds it freezes when launching a game.
- Disabled a Shogun 2 error message that was spamming the crash report service.

### Fixed
- Fixed some situations where Empire and Napoleon would start without mods enabled.
- Fixed default update channel sometimes getting saved wrong.

## [0.9.9]
### Fixed
- Fixed some cornercases of modded single-entity units not being treated as such by the unit multiplier.
- Fixed some crashes with launch options enabled in older games.
- Fixed mods with movie packs on data not being clickable.

## [0.9.8]
### Fixed
- Fixed many instances where incorrect/duplicated paths were added to the mod list file.
- Fixed profile selected resetting after saving it.

## [0.9.7]
### Fixed
- Fixed CTD on Empire and Napoleon when using launch options.
- Fixed language not being detected correctly by the launch options in certain games.

## [0.9.6]
### Fixed
- Fixed game crash when using some launch options and not using the unit multiplier on heavy load orders.
- Fixed autotranslator mistranslating entries of mods with loc files outside text/. 
- Fixed an issue where some custom load orders will cause unexpected behavior ingame when using launch options. 

## [0.9.5]
### Added
- Implemented support for Pharaoh's Dynasties update.

### Changed
- When uploading a mod to the workshop, Runcher will automatically alter the pack so Steam doesn't reject the update due to no changes. This means there's no need anymore to re-save the pack after a patch in order to upload it to remove the "outdated" mark.

## [0.9.4]
### Fixed
- Fixed mods not loading when launching Empire with mods enabled.

## [0.9.3]
### Added
- Implemented prototype of Universal Rebalancer (only for WH3).

### Changed
- Icons for non-installed games are now hidden by default.
- Optimized launch option's processing to take a fraction of the time they took before.

### Fixed
- Fixed Runcher hanging when workshopper failed to execute in a previous launch.
- Fixed Runcher passing incorrect args to the game when there are no mods in the load order.
- Fixed a few situations where Runcher called Workshopper without correct args.

## [0.9.2]
### Fixed
- Fixed double-update when restarting Runcher after an update.
- Fixed changelog not opening after an update.
- Fixed launch options causing certain mods to not apply certain tables as the game expected them.
- Fixed translations launch option sometimes trying to re-apply an already applied translation.
- Whitelisted PJ scripts from the lua error checker due to being intentional errors.

## [0.9.1]
### Fixed
- Fixed typos in error messages.
- Fixed "Invalid path" and other related errors on new installs.
- Fixed some launch options for shogun 2 not getting properly initialized.

## [0.9.0]
### Added
- Implemented support for Shogun 2 Map Mods.
- Implemented support for forcing a download of all subscribed mods (doesn't work on legacy mods).
- Implemented support for forcing a download of selected mods.
- Implemented post-launch log analyzer.
- Implemented "Remove Trait Limit" launch option for Warhammer 3.
- Implemented setting to block steam from updating the a game so people can finish their campaigns.
- Added pack location to the mod list.
- Implemented flags for outdated packs which are overriding more recent workshop packs.

### Changed
- The terminal should no longer show up for a second when launching a game.
- Double-clicking a file in the data view will open it in RPFM, if RPFM is configured as a Tool.

### Fixed
- Fixed CTD when a very particular set of occurences happened with Shogun 2 as Game Selected.
- Fixed a bunch of situations where path checks were not done correctly due to missing canonicalization of paths.
- Fixed a few situations where Shogun 2 mods may become detached from their main mod, appearing as two mods in the mod list.
- Fixed a few situations where Shogun 2 mods may end up showing up in the secondary folder as .bin files.
- Fixed Workshopper failing to get mods information if one of the Published File Ids was non-numeric.
- Fixed incorrect Published File Id being detected for packs in content sub-subfolders.
- Fixed user steam id not being retrieved if the user did not installed Steam in C.
- Fixed possible bug where multiple patchers patching the same table fail to patch one over the other.
- Fixed steamworks integration not working for some commands when the command ended up being longer than 8191 characters.

## [0.8.2]
### Added
- Implemented support for secondary folder and temp packs for Shogun 2.
- Implemented unit multiplier support for Three Kingdoms.

### Changed
- Changelog should now automatically open after an update.

### Fixed
- Fixed hangs and freezes when the steamworks api breaks and starts returning errors.
- Fix rare situation where an breaking error will be logged but the program would not actually panic, staying open in a broken state.
- Fixed translations not properly restoring lines that were in the vanilla english locs, but not in the translated vanilla locs, or were in the translated vanilla locs but they were empty.
- Fixed unit multiplier causing Tomb Kings Realm of Souls to trigger later than expected.

## [0.8.1]
### Added
- Implemented support for YYYY/MM/DD dates.

### Changed
- Reworked how games are initialized. This should fix all instances of games starting without recognizing their DLCs.
- Shogun 2 no longer has it's launcher replaced by a custom exe.

### Fixed
- Fixed Empire not being visible by default in the game list.

## [0.8.0]
### Added
- Implemented load-order dependant data view.
- Implemented "Expand/Collapse All" actions in the mod list.
- Implemented "Open Workshop Links on Steam" setting. 
- Implemented "Secondary Mods Folder" feature.
- Implemented "Copy/Move to Secondary" feature.
- Implemented "Import ModList" feature.
- Implemented "Workshopper" companion program for interaction with the SteamWorks API.
- Implemented "Upload/Update to Workshop" feature.
- Implemented support for toggling Movie packs not in /data.

### Changed
- When renaming a category, its current name is the default value in the dialog.
- Network request done while loading a game's data (requesting steam workshop info about the downloaded mods) are now done "partially async". 
    + This means starting the program or changing the game selected takes a few less seconds.
- Steam Workshop data request (for getting mod info from the workshop) has been moved to use the SteamWorks API.
- Runcher now requires Steam to be running in order to start.

### Fixed
- Fixed mysterious error when launching Runcher with no game path configured.
- Fixed actions being enabled when a non-installed game is selected (like on new installs or installs with a default game selected with a missing/invalid path).
- Fixed creation time error when running runcher on filesystems that do not support creation times.
- Fixed possible CTDs when importing a load order from a string.
- Fixed possible CTDs when exporting a load order into a string.
- Fixed hang when changing a path in the settings.
- Fixed a myriad of random issues in the import/export loadorder code.
- Fixed unit multiplier incorrectly marking units as SEM units.
- Fixed unit multiplier causing chariots to have random horses inside them.
- Fixed unit multiplier causing chariots to be empty of units due to rounding errors.
- Fixed unit multiplier causing units to sometimes break when they're edited in a mod.
- Fixed translator not automatically translating unchanged english text in games that do not use the old multilanguage logic.

## [0.7.0]
### Added
- Implemented custom font support.
- Implemented support for the following start args:
    + game: allows you to start Runcher with a game selected different of the default one.
    + profile: allows you to auto-load a profile for a specific game when starting Runcher.
    + autostart: allows you to skip the UI and start the game directly. 
        * Combined with game and profile, this allows users to make profile-specific shortcuts.
- Implemented manual load order support.
- Implemented profile manager.
- Implemented "Sort Category" feature, to help speedup setting up in the new update.
- Implemented "Open With Tools" feature.

### Changed
- Empty categories are now remembered even if they have no mods.
- Mods deleted and reinstalled no longer end up back in their old category.
- Unassigned category is now always last.
- Mods can now be reordered and moved between categories by dragging and dropping them.
- Categories can now be reordered to your hearts contend by dragging and dropping them.
- Packs in the load order can now be reordered to your hearts contend by dragging and dropping them.
- Optimized toggling large amounts of mods at once.
- Profiles now remember if the load order is in automatic or manual mode.
- Runcher will now make sure to clean it's reserved packs to avoid weird behavior in updates.
- "Enable Translations" launch option will now make sure your language has priority over mods that have english lines unchanged from vanilla.
- "Enable Translations" launch option now can auto-apply unofficial fixes for vanilla translations.

### Fixed
- Fixed dark theme not reloading correctly after toggling it.
- Fixed update folders not getting cleanup on start.
- Fixed decoding error when loading a game which saves are not yet supported to be decoded.
- Fixed incorrect updating messages.
- Fixed clean installs breaking the mod list.
- Fixed broken profiles causing the entire program to hang at boot.
- Fixed "Send To Category" not working properly when multiple items are selected.
- Fixed load order being calculated wrong due to a bug in RPFM.
- Fixed vanilla-file-processing operations being done wrong due to a bug in RPFM. 
- Fixed main window hanging when changing game selected. 
- Fixed CTD under some circustances when skipping intros. 
- Fixed Pharaoh's skip intro not skipping epilepsy warning. 
- Fixed wrong load order calculation when a "legacy" bin mod is involved in the process. 

## [0.6.0]
### Added
- Implemented "Enable translations" feature for all games (fixes the "no text on not-english language" bug in old games).
- Implemented "Enable Logging" feature for:
    + Warhammer 3
    + Warhammer 2
    + Troy
- Implemented "Skip Intro" feature for all games.
- Implemented "Open Game Config Folder" button.
- Implemented github, discord and patreon buttons on the bottom right of the screen (like in RPFM).
- Implemented support for Pharaoh.
- Implemented setting to change date format between the logical one and the american one.
- Implemented "Update Manager", and merged all "Check XXX Updates" options into it.

### Changed
- Game selected menu moved to a left-side toolbar.
- Refactored a large part of the codebase to make it more flexible and less prone to bugs.
- Runcher should no longer throw a flash grenade to you when you open it.
- Runcher should no longer allow you to select a game you don't have installed.
- Runcher should no longer allow you to select "Open Contents folder" and "Save" if the game doesn't support them.
- Reworked launch options menu to look less ugly.
- About menu buttons have been moved to the bottom right of the screen.

### Removed
- "About Qt" button has been removed.
- "About" menu has been removed.

### Fixed
- Fixed Runcher not remembering the window geometry after closing it.
- Fixed CTD when a non-installed game is selected.
- Fixed unit multiplier being available without schema.
- Fixed saves not getting cleared after triggering a game selected change.
- Fixed schema update not triggering a full game selected reload.
- Fixed CTD when launching a game if you don't have the exe.
- Fixed several issues when switching the game selected.
- Fixed unit multiplier not working under certain circustances.
- Fixed multiple instances of vanilla movie packs showing up as mods.
- Fixed launch options sometimes not getting correctly saved.
- Fixed extra pack not loading for certain games.
- Fixed movies.pack not being marked as vanilla file in empire.
- Fixed userscript edits not woking for shogun 2, empire and napoleon.
- Fixed error when trying to launch a game that has not been launched before.
- Fixed error when trying to launch a game with no mods while using the unit multiplier.
- Fixed multiple issues causing all games older than warhammer to work partially or not work at all.
- Fixed runcher trying to load mods from content for games that do not support loading mods from outside the /data folder.
- Fixed launch options menu closing after clicking on a checkbox.
- Fixed Shogun 2 mods not being correctly copied from /content if they ended in .pack.

## [0.5.0]
### Added
- Implemented unit multiplier for WH3.
- Implemented schema downloader.
- Implemented load to save support.

### Fixed
- Fixed support for .bin mods, shogun 2 content mods, and pre-rome 2 mods in general.
- Fixed launch options being available for games where they don't work.
- Fixed some issues causing saves to complain about missing "run_you_fools" pack.

## [0.4.3]
### Added
- Implemented movie pack detection support.
- Implemented outdated mod detection.
- Performance improvements when starting/refreshing.
- Updated `Skip Intro Movies` support for WH3 3.0.

### Fixed
- Fixed incorrect icons on dark theme.
- Fixed CTD when opening Runcher on a computer with no Steam installation.
- Fixed incorrect icons on dark theme due to missing dlls.
- Fixed incorrect sorting on date columns.
- Fixed incorrect text on settings.
- Fixed script logger not working.

## [0.4.2]
### Fixed
- Fixed incorrect icons on dark theme.
- Fixed CTD when opening Runcher on a computer with no Steam installation.

## [0.4.1]
### Added
- Implemented autodetection for steam-installed games.
- Implemented "Enable/Disable selection" feature.

### Changed
- Improved dark theme.

### Fixed
- Fixed profiles not loading properly.

## [0.4.0]
### Added
- Implemented "Skip Intros" for Warhammer 3.
- Implemented "Enable Logging" for Warhammer 3.

### Changed
- Reduced release size.

### Fixed
- Fixed missing icons on debug builds.
- Fixed vanishing expander on mod list.
- Fixed incorrect size calculation for Size column.
- Fixed unsorted categories on "Send to Category" list.

## [0.3.1]
### Fixed
- Fixed weird issues due to incorrect logic when generating mod list file.

## [0.3.0]
### Added
- Implemented "Check Updates on Start" setting.
- Implemented "Copy/Paste Load Order" features.
- Implemented "Rename Category" feature.

### Fixed
- Fixed Tree Views not getting reloaded correctly.
- Fixed folder Tool Button not showing its menu when pressed. 
- Fixed "Check Updates" hanging the main thread.
- Fixed weird resizing issues.
- Fixed incorrect visual pack order.

## [0.2.0]
### Added
- Implemented "Open Folder" actions for commonly used folders.
- Implemented "Open In Explorer" feature.
- Implemented "Open In Steam Workshop" feature.
- Implemented multilanguage support.
- Implemented dark theme support.
- Implemented "Reload" feature.
- Added more columns with mod data to the mod list.
- Added support for passing the game selected as an argument.

### Changed
- Pack name is now shown beside the mod name for steam mods.
- Mod list is now sortable.
- Mod list is now resizable.

### Fixed
- Fixed a few settings not being remembered between sessions.
- Fixed mod list not being cleared on reload.

## [0.1.1]
### Changed
- Location column now shows the Steam Id for Content mods.

### Fixed
- Fixed mod context menu allowing actions with invalid selections.
- Fixed deleting categories not triggering a game-config save.
- Fixed deleting categories not working for multiple categories.
- Fixed deleting categories removing mods from the list until a reload was triggered.
- Fixed send to category not working for multiple mods.
- Fixed content mods crashing the game.
- Fixed CTDs not being reported.

## [0.1.0]
### Added
- Initial release.

[Unreleased]: https://github.com/Frodo45127/runcher/compare/v0.9.16...HEAD
[0.9.16]: https://github.com/Frodo45127/runcher/compare/v0.9.15...v0.9.16
[0.9.15]: https://github.com/Frodo45127/runcher/compare/v0.9.14...v0.9.15
[0.9.14]: https://github.com/Frodo45127/runcher/compare/v0.9.13...v0.9.14
[0.9.13]: https://github.com/Frodo45127/runcher/compare/v0.9.12...v0.9.13
[0.9.12]: https://github.com/Frodo45127/runcher/compare/v0.9.11...v0.9.12
[0.9.11]: https://github.com/Frodo45127/runcher/compare/v0.9.10...v0.9.11
[0.9.10]: https://github.com/Frodo45127/runcher/compare/v0.9.9...v0.9.10
[0.9.9]: https://github.com/Frodo45127/runcher/compare/v0.9.8...v0.9.9
[0.9.8]: https://github.com/Frodo45127/runcher/compare/v0.9.7...v0.9.8
[0.9.7]: https://github.com/Frodo45127/runcher/compare/v0.9.6...v0.9.7
[0.9.6]: https://github.com/Frodo45127/runcher/compare/v0.9.5...v0.9.6
[0.9.5]: https://github.com/Frodo45127/runcher/compare/v0.9.4...v0.9.5
[0.9.4]: https://github.com/Frodo45127/runcher/compare/v0.9.3...v0.9.4
[0.9.3]: https://github.com/Frodo45127/runcher/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/Frodo45127/runcher/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/Frodo45127/runcher/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/Frodo45127/runcher/compare/v0.8.2...v0.9.0
[0.8.2]: https://github.com/Frodo45127/runcher/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/Frodo45127/runcher/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/Frodo45127/runcher/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/Frodo45127/runcher/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Frodo45127/runcher/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Frodo45127/runcher/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/Frodo45127/runcher/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/Frodo45127/runcher/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/Frodo45127/runcher/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Frodo45127/runcher/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Frodo45127/runcher/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Frodo45127/runcher/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Frodo45127/runcher/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Frodo45127/runcher/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Frodo45127/runcher/compare/...v0.1.0
