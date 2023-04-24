# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

If you're looking for the changes included in the latest beta (against the latest stable version), check the unreleased section.

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

[Unreleased]: https://github.com/Frodo45127/runcher/compare/v0.4.3...HEAD
[0.4.3]: https://github.com/Frodo45127/runcher/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/Frodo45127/runcher/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/Frodo45127/runcher/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Frodo45127/runcher/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Frodo45127/runcher/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Frodo45127/runcher/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Frodo45127/runcher/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Frodo45127/runcher/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Frodo45127/runcher/compare/...v0.1.0
