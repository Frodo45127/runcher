#include "mod_list_model.h"

extern "C" QStandardItemModel* new_mod_list_model() {
    return dynamic_cast<QStandardItemModel*>(new ModListModel());
}

// Function to check if an item can be drag or drop into.
Qt::ItemFlags ModListModel::flags(const QModelIndex &index) const {
    Qt::ItemFlags defaultFlags = QStandardItemModel::flags(index);
    defaultFlags = defaultFlags &~ Qt::ItemIsDragEnabled;
    defaultFlags = defaultFlags &~ Qt::ItemIsDropEnabled;

    // In case of valid index, allow:
    // - Drag for everything.
    // - Drop for cats only.
    if (index.isValid()) {
        QStandardItem* item = itemFromIndex(index);
        bool isCat = item->data(40).toBool();
        if (isCat) {
            return Qt::ItemIsDragEnabled | Qt::ItemIsDropEnabled | defaultFlags;
        }
        else {
            return Qt::ItemIsDragEnabled | defaultFlags;
        }
    }

    // In case of invalid index, allow dropping.
    else {
        return Qt::ItemIsDropEnabled | defaultFlags;
    }
}
