#ifndef LAUNCHERWINDOW_H
#define LAUNCHERWINDOW_H

#include <QApplication>
#include <QCloseEvent>
#include <QDebug>
#include <QFileInfo>
#include <QIcon>
#include <QResource>
#include <QMainWindow>

extern "C" QMainWindow* launcher_window(bool use_dark_theme);

class LauncherWindow: public QMainWindow {
    Q_OBJECT
public:
    explicit LauncherWindow(QWidget *parent = nullptr, bool use_dark_theme = false);
    void closeEvent(QCloseEvent *event) override;
};

#endif // LAUNCHERWINDOW_H
