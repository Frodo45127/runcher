# Remember to execute this from the root of RPFM's git folder.
Set-Variable -Name "RUNCHER_PATH" -Value ((Get-Location).path)
Set-Variable -Name "RUNCHER_VERSION" -Value (Select-String -Path Cargo.toml -Pattern '^version = \"(.*)\"$').Matches.Groups[1].value

# Build the tools.
cargo build --release

# Prepare the paths for the deployment.
Set-Location I:\
Remove-Item -r -fo I:\deploy
mkdir deploy
Set-Location deploy
mkdir runcher-release-assets
Set-Location runcher-release-assets

# Copy Breeze icons into the release.
mkdir -p data/icons
Copy-Item "C:\CraftRoot\bin\data\icons\breeze" "I:\deploy\runcher-release-assets\data\icons\" -recurse
Copy-Item "C:\CraftRoot\bin\data\icons\breeze-dark" "I:\deploy\runcher-release-assets\data\icons\" -recurse

# Here we copy all the dlls required by runcher. Otherwise we'll have to manually update them on every freaking release, and for 2 months that's been a royal PITA.
mkdir designer
Copy-Item C:\CraftRoot\plugins\designer\*.dll I:\deploy\runcher-release-assets\designer\

mkdir iconengines
Copy-Item C:\CraftRoot\plugins\iconengines\KIconEnginePlugin.dll I:\deploy\runcher-release-assets\iconengines\
Copy-Item C:\CraftRoot\plugins\iconengines\qsvgicon.dll I:\deploy\runcher-release-assets\iconengines\

mkdir imageformats
Copy-Item C:\CraftRoot\plugins\imageformats\*.dll I:\deploy\runcher-release-assets\imageformats\

mkdir platforms
Copy-Item C:\CraftRoot\plugins\platforms\qwindows.dll I:\deploy\runcher-release-assets\platforms\

mkdir styles
Copy-Item C:\CraftRoot\plugins\styles\qwindowsvistastyle.dll I:\deploy\runcher-release-assets\styles\

Copy-Item C:\CraftRoot\bin\d3dcompiler_47.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\dbus-1-3.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\editorconfig.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\freetype.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\harfbuzz.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\iconv.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\icudt??.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\icuin??.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\icuuc??.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\intl-8.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\jpeg62.dll I:\deploy\runcher-release-assets\

Copy-Item C:\CraftRoot\bin\KF5Archive.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5Codecs.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5ConfigCore.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5ConfigGui.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5ConfigWidgets.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5CoreAddons.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5GuiAddons.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5I18n.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5IconThemes.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\KF5WidgetsAddons.dll I:\deploy\runcher-release-assets\

Copy-Item C:\CraftRoot\bin\libbzip2.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\libcrypto*.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\libEGL.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\libGLESV2.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\liblzma.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\libpng16.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\libssl*.dll I:\deploy\runcher-release-assets\

# Are these still neccesary?
Copy-Item C:\CraftRoot\bin\msvcp140.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\msvcp140_1.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\msvcp140_2.dll I:\deploy\runcher-release-assets\

Copy-Item C:\CraftRoot\bin\pcre2-8.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\pcre2-16.dll I:\deploy\runcher-release-assets\

Copy-Item C:\CraftRoot\bin\Qt5DBus.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5Svg.dll I:\deploy\runcher-release-assets\

# Same as before. Still neccesary?
Copy-Item C:\CraftRoot\bin\vcruntime140.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\vcruntime140_1.dll I:\deploy\runcher-release-assets\

Copy-Item C:\CraftRoot\bin\tiff.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\zlib1.dll I:\deploy\runcher-release-assets\
Copy-Item C:\CraftRoot\bin\zstd.dll I:\deploy\runcher-release-assets\

# Here we copy files generated from the compilation.
Copy-Item $RUNCHER_PATH/target/release/runcher.exe I:\deploy\runcher-release-assets
Copy-Item $RUNCHER_PATH/target/release/runcher.pdb I:\deploy\runcher-release-assets

# Workshopper for workshop and steam launch support.
Copy-Item $RUNCHER_PATH/workshopper/3rdparty/steam_api64.dll I:\deploy\runcher-release-assets
Copy-Item $RUNCHER_PATH/target/release/workshopper.exe I:\deploy\runcher-release-assets

# TWPatcher for load order patching support.
Copy-Item $RUNCHER_PATH/target/release/twpatcher.exe I:\deploy\runcher-release-assets

# Here we copy assets from the repo.
mkdir icons
mkdir locale
mkdir ui
Copy-Item $RUNCHER_PATH/LICENSE I:\deploy\runcher-release-assets
Copy-Item $RUNCHER_PATH/CHANGELOG.md I:\deploy\runcher-release-assets
Copy-Item $RUNCHER_PATH/CHANGELOG.md I:\deploy\runcher-release-assets\CHANGELOG.txt
Copy-Item $RUNCHER_PATH/dark-theme.qss I:\deploy\runcher-release-assets
Copy-Item $RUNCHER_PATH/icons/* I:\deploy\runcher-release-assets\icons\
Copy-Item $RUNCHER_PATH/locale/* I:\deploy\runcher-release-assets\locale\
Copy-Item $RUNCHER_PATH/ui_templates/* I:\deploy\runcher-release-assets\ui\

# Execute windeployqt to add missing translations and the vcredist if needed.
windeployqt runcher.exe

# Remove extra files that are not really needed for execution.
Remove-Item -fo I:\deploy\runcher-release-assets\vc_redist.x64.exe
Remove-Item -fo I:\deploy\runcher-release-assets\icons\breeze-icons.rcc
Remove-Item -fo I:\deploy\runcher-release-assets\icons\breeze-icons-dark.rcc

Set-Location I:\deploy
7z a runcher-$RUNCHER_VERSION-x86_64-pc-windows-msvc.zip .\**

# Move back to the original folder.
Set-Location $RUNCHER_PATH
