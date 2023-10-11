# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

If you're looking for the changes included in the latest beta (against the latest stable version), check the unreleased section.

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

[Unreleased]: https://github.com/Frodo45127/runcher/compare/v0.6.0...HEAD
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
