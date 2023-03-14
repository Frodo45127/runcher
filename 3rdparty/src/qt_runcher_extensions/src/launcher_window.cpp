#include "kicontheme.h"
#include "launcher_window.h"

// Fuction to be able to create a custom QMainWindow.
extern "C" QMainWindow* launcher_window() {
    return dynamic_cast<QMainWindow*>(new LauncherWindow(nullptr));
}

LauncherWindow::LauncherWindow(QWidget *parent) : QMainWindow(parent) {
    #ifdef _WIN32

        // Initialize the icon theme. Holy shit this took way too much research to find how it works.
        const QString iconThemeName = QStringLiteral("breeze");

        const QString iconThemeRccFallback = qApp->applicationDirPath() + QStringLiteral("/data/icons/breeze/breeze-icons.rcc");

        qWarning() << "Rcc file for Light theme" << iconThemeRccFallback;

        if (!iconThemeRccFallback.isEmpty()) {
            const QString iconSubdir = QStringLiteral("/icons/") + iconThemeName;
            bool load_fallback = QResource::registerResource(iconThemeRccFallback, iconSubdir);

            // If nothing failed, set the themes.
            if (load_fallback) {
                if (QFileInfo::exists(QLatin1Char(':') + iconSubdir + QStringLiteral("/index.theme"))) {
                    QIcon::setThemeName(iconThemeName);
                    QIcon::setFallbackThemeName(QStringLiteral("breeze"));
                } else {
                    qWarning() << "No index.theme found in" << iconThemeRccFallback;
                    QResource::unregisterResource(iconThemeRccFallback, iconSubdir);
                }
            } else {
                qWarning() << "Invalid rcc file" << iconThemeRccFallback;
            }
        } else {
            qWarning() << "Empty rcc file" << iconThemeRccFallback;
        }
    #endif
}
