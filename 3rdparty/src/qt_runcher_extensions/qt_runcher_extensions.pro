#-------------------------------------------------
#
# Project created by QtCreator 2018-09-28T23:28:57
#
#-------------------------------------------------

QT       += widgets
QT       += KIconThemes
QT       += KTextEditor
QT       += KWidgetsAddons
QT       += KCompletion

TARGET = qt_runcher_extensions
TEMPLATE = lib

# We only want the release version, as this lib is not going to get "advanced" stuff.
# In case you want to build the debug version, change the following line, removing the "release".
CONFIG += staticlib release

DEFINES += QT_RUNCHER_EXTENSIONS_LIBRARY

# The following define makes your compiler emit warnings if you use
# any feature of Qt which has been marked as deprecated (the exact warnings
# depend on your compiler). Please consult the documentation of the
# deprecated API in order to know how to port your code away from it.
DEFINES += QT_DEPRECATED_WARNINGS

# You can also make your code fail to compile if you use deprecated APIs.
# In order to do so, uncomment the following line.
# You can also select to disable deprecated APIs only up to a certain version of Qt.
DEFINES += QT_DISABLE_DEPRECATED_BEFORE=0x060000    # disables all the APIs deprecated before Qt 6.0.0

SOURCES += \
    src/launcher_window.cpp
    #src/packed_file_model.cpp \
    #src/treeview_draggable.cpp \
    #src/treeview_filter.cpp

HEADERS += \
    src/launcher_window.h

INCLUDEPATH += include

release:DESTDIR = release
release:OBJECTS_DIR = release/.obj
release:MOC_DIR = release/.moc
release:RCC_DIR = release/.rcc
release:UI_DIR = release/.ui

debug:DESTDIR = debug
debug:OBJECTS_DIR = debug/.obj
debug:MOC_DIR = debug/.moc
debug:RCC_DIR = debug/.rcc
debug:UI_DIR = debug/.ui

windows {
    INCLUDEPATH += C:/CraftRoot/include

    # Fix for the broken KSyntaxHighlighting include on linux, by AaronBPaden.
    INCLUDEPATH += C:/CraftRoot/include/KF5/KSyntaxHighlighting

    # Same fix for the build machine.
    INCLUDEPATH += D:/Craft/include/KF5/KSyntaxHighlighting
}

unix {

    # For some reason, these flags fuck up compilation on windows, so we leave them linux only.
    QMAKE_CXXFLAGS = -Wl,-rpath='${ORIGIN}'

    # Fix for the broken KSyntaxHighlighting include on linux, by AaronBPaden.
    INCLUDEPATH += /usr/include/KF5/KSyntaxHighlighting
}

# This means we generate all the artifacts in target and drop the final lib in libs.
DESTDIR         = ../../builds
BASEDIR         = ../../../target/qt_runcher_extensions
MOC_DIR         = ../../../target/qt_runcher_extensions/moc
OBJECTS_DIR     = ../../../target/qt_runcher_extensions/obj

# Fix for make failing due to missing folders.
commands = ; $(MKDIR -p) BASEDIR; $(MKDIR -p) $MOC_DIR; $(MKDIR -p) $OBJECTS_DIR
