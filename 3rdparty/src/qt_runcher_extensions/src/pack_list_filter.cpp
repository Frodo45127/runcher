#include "pack_list_filter.h"
#include <QItemSelection>
#include <QRegExp>
#include <QStandardItemModel>

// Function to create the filter in a way that we don't need to bother Rust with new types.
extern "C" QSortFilterProxyModel* pack_list_filter(QObject *parent) {
    PackListFilter* filter = new PackListFilter(parent);
    return dynamic_cast<QSortFilterProxyModel*>(filter);
}

// Function to trigger the filter we want, instead of the default one, from Rust.
extern "C" void pack_list_trigger_filter(QSortFilterProxyModel* filter, QRegExp* pattern) {
    PackListFilter* filter2 = static_cast<PackListFilter*>(filter);
    filter2->setFilterRegExp(*pattern);
}

// Constructor of QTreeViewSortFilterProxyModel.
PackListFilter::PackListFilter(QObject *parent): QSortFilterProxyModel(parent) {}

// Function called when the filter changes.
bool PackListFilter::filterAcceptsRow(int source_row, const QModelIndex &source_parent) const {

    // Check the current item. If it's a file, we just call the parent's filter.
    bool result = QSortFilterProxyModel::filterAcceptsRow(source_row, source_parent);

    // Check the siblings too for pack name filtering.
    for (int i = 1; i < sourceModel()->columnCount() && !result; ++i) {
        QModelIndex sibIndex = sourceModel()->index(source_row, i, source_parent);
        QString sibData = sibIndex.data(Qt::DisplayRole).toString();
        if (!sibData.isEmpty()) {
            result |= sibData.contains(filterRegExp());
        }
    }

    QModelIndex currntIndex = sourceModel()->index(source_row, 0, source_parent);

    // If it has children, is a folder, so check each of his children.
    if (sourceModel()->hasChildren(currntIndex)) {
        for (int i = 0; i < sourceModel()->rowCount(currntIndex) && !result; ++i) {

            // Keep the parent if a children is shown.
            result |= filterAcceptsRow(i, currntIndex);
        }
    }

    // If it's a file, and it's not visible, there is a special behavior:
    // if the parent matches the filter, we assume all it's children do it too.
    // This is to avoid the "Show table folder, no table file" problem.
    else if (!result) {
        QModelIndex granpa = source_parent.parent();
        int granpa_row = source_parent.row();
        result = QSortFilterProxyModel::filterAcceptsRow(granpa_row, granpa);
    }

    return result;
}
