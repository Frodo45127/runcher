#include "mod_list_filter.h"
#include <QItemSelection>
#include <QRegExp>
#include <QSortFilterProxyModel>
#include <QStandardItem>
#include <QStandardItemModel>

extern "C" QSortFilterProxyModel* mod_list_filter(QObject *parent) {
    ModListFilter* filter = new ModListFilter(parent);
    return dynamic_cast<QSortFilterProxyModel*>(filter);
}

extern "C" void mod_list_trigger_filter(QSortFilterProxyModel* filter, QRegExp* pattern) {
    ModListFilter* filter2 = static_cast<ModListFilter*>(filter);
    filter2->setFilterRegExp(*pattern);
}

ModListFilter::ModListFilter(QObject *parent): QSortFilterProxyModel(parent) {}

bool ModListFilter::filterAcceptsRow(int source_row, const QModelIndex &source_parent) const {
    bool result = QSortFilterProxyModel::filterAcceptsRow(source_row, source_parent);
    QModelIndex currntIndex = sourceModel()->index(source_row, 0, source_parent);
    bool isCategory = currntIndex.data(40).toBool();

    // Always show categories.
    if (isCategory) {
        result |= true;
    }

    return result;
}

void ModListFilter::sort(int column, Qt::SortOrder order) {
    if (column == 6 || column == 7 || column == 8) {
        setSortRole(30);
        QSortFilterProxyModel::sort(column, order);
    } else {
        setSortRole(2);
        QSortFilterProxyModel::sort(column, order);
    }
}
