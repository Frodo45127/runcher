#include "mod_list_tree_view.h"

#include <QMimeData>
#include <QStandardItem>
#include <QHeaderView>

extern "C" QTreeView* new_mod_list_tree_view(QWidget *parent) {
    return dynamic_cast<QTreeView*>(new ModListTreeView(parent));
}

ModListTreeView::ModListTreeView(QWidget *parent) : QTreeView(parent) {
    setContextMenuPolicy(Qt::CustomContextMenu);
    setAlternatingRowColors(true);
    setSelectionMode(SelectionMode::ExtendedSelection);
    setSelectionBehavior(QAbstractItemView::SelectionBehavior::SelectRows);

    setUniformRowHeights(true);
    setSortingEnabled(false);
    setAnimated(true);
    setAllColumnsShowFocus(true);
    setHeaderHidden(false);
    setExpandsOnDoubleClick(true);
    header()->setVisible(true);
    header()->setStretchLastSection(true);

    setDragEnabled(true);
    setAcceptDrops(true);
    setDropIndicatorShown(true);
    setDragDropMode(DragDropMode::InternalMove);
    setDragDropOverwriteMode(false);

    setRootIndex(QModelIndex());
}

void ModListTreeView::dragEnterEvent(QDragEnterEvent *event) {
    QTreeView::dragEnterEvent(event);
}

void ModListTreeView::dragMoveEvent(QDragMoveEvent *event) {
    QTreeView::dragMoveEvent(event);
}

void ModListTreeView::dragLeaveEvent(QDragLeaveEvent *event) {
    QTreeView::dragLeaveEvent(event);
}

void ModListTreeView::dropEvent(QDropEvent *event) {
    QModelIndex index = indexAt(event->pos());
    if (!index.isValid()) {
        return;
    }

    QModelIndex parent = index.parent();

    // NOTE, because I forgot how this works. This rejects all drops, but emits a signal so we can
    // perform the move manually in rust, where we can check if the move is valid more accuratelly.
    emit itemDrop(parent, index.row());
}
