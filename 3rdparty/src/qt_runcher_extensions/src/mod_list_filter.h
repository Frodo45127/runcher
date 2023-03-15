#ifndef MOD_LIST_FILTER_H
#define MOD_LIST_FILTER_H

#include <QSortFilterProxyModel>

extern "C" QSortFilterProxyModel* mod_list_filter(QObject *parent = nullptr);
extern "C" void mod_list_trigger_filter(QSortFilterProxyModel *filter = nullptr, QRegExp* pattern = nullptr);

class ModListFilter : public QSortFilterProxyModel {
    Q_OBJECT

public:
    explicit ModListFilter(QObject *parent = nullptr);
    bool filterAcceptsRow(int source_row, const QModelIndex & source_parent) const;
};

#endif // MOD_LIST_FILTER_H
