#ifndef LAUNCHERWINDOW_H
#define LAUNCHERWINDOW_H

#include <QApplication>
#include <QDebug>
#include <QFileInfo>
#include <QIcon>
#include <QResource>
#include <QMainWindow>

extern "C" QMainWindow* launcher_window();

class LauncherWindow: public QMainWindow {
    Q_OBJECT
public:
    explicit LauncherWindow(QWidget *parent = nullptr);
};

#endif // LAUNCHERWINDOW_H
