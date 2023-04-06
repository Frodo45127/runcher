#include "kicontheme.h"
#include "launcher_window.h"

// Fuction to be able to create a custom QMainWindow.
extern "C" QMainWindow* launcher_window(bool use_dark_theme) {
    return dynamic_cast<QMainWindow*>(new LauncherWindow(nullptr, use_dark_theme));
}

LauncherWindow::LauncherWindow(QWidget *parent, bool use_dark_theme) : QMainWindow(parent) {
    #ifdef _WIN32

        // Initialize the icon theme. Holy shit this took way too much research to find how it works.
        const QString iconThemeName = QStringLiteral("breeze");

        const QString iconThemeRccFallback = qApp->applicationDirPath() + QStringLiteral("/data/icons/breeze/breeze-icons.rcc");
        const QString iconThemeRccDark = qApp->applicationDirPath() + QStringLiteral("/data/icons/breeze-dark/breeze-icons-dark.rcc");

        qWarning() << "Rcc file for Dark theme" << iconThemeRccDark;
        qWarning() << "Rcc file for Light theme" << iconThemeRccFallback;

        if (!iconThemeRccDark.isEmpty() && !iconThemeRccFallback.isEmpty()) {
            const QString iconSubdir = QStringLiteral("/icons/") + iconThemeName;
            bool load_fallback = QResource::registerResource(iconThemeRccFallback, iconSubdir);

            // Only load the dark theme resources if needed.
            bool load_dark = false;
            if (dark_theme_enabled) {
                load_dark = QResource::registerResource(iconThemeRccDark, iconSubdir);
            }

            // If nothing failed, set the themes.
            if (load_fallback && (load_dark || !dark_theme_enabled)) {
                if (QFileInfo::exists(QLatin1Char(':') + iconSubdir + QStringLiteral("/index.theme"))) {
                    QIcon::setThemeName(iconThemeName);
                    QIcon::setFallbackThemeName(QStringLiteral("breeze"));
                } else {
                    qWarning() << "No index.theme found in" << iconThemeRccDark;
                    qWarning() << "No index.theme found in" << iconThemeRccFallback;
                    QResource::unregisterResource(iconThemeRccDark, iconSubdir);
                    QResource::unregisterResource(iconThemeRccFallback, iconSubdir);
                }
            } else {
                qWarning() << "Invalid rcc file" << iconThemeRccFallback;
            }
        } else {
            qWarning() << "Empty rcc file" << iconThemeRccDark;
            qWarning() << "Empty rcc file" << iconThemeRccFallback;
        }
    #endif
}
