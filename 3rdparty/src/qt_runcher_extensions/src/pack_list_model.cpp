#include "pack_list_model.h"

extern "C" QStandardItemModel* new_pack_list_model() {
    return dynamic_cast<QStandardItemModel*>(new PackListModel());
}

// Function to check if an item can be drag or drop into.
Qt::ItemFlags PackListModel::flags(const QModelIndex &index) const {
    Qt::ItemFlags defaultFlags = QStandardItemModel::flags(index);
    defaultFlags = defaultFlags &~ Qt::ItemIsDragEnabled;
    defaultFlags = defaultFlags &~ Qt::ItemIsDropEnabled;

    if (index.isValid()) {
        QStandardItem* item = itemFromIndex(index);
        return Qt::ItemIsDragEnabled | defaultFlags;
    }

    // In case of invalid index, allow dropping.
    else {
        return Qt::ItemIsDropEnabled | defaultFlags;
    }
}
