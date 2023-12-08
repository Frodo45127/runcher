#ifndef PACK_LIST_FILTER_H
#define PACK_LIST_FILTER_H

#include <QSortFilterProxyModel>
#include <QStandardItem>

extern "C" QSortFilterProxyModel* pack_list_filter(QObject *parent = nullptr);
extern "C" void pack_list_trigger_filter(QSortFilterProxyModel *filter = nullptr, QRegExp* pattern = nullptr);

class PackListFilter : public QSortFilterProxyModel
{
    Q_OBJECT

public:

    explicit PackListFilter(QObject *parent = nullptr);
    bool filterAcceptsRow(int source_row, const QModelIndex & source_parent) const;

signals:

private:
};

#endif // PACK_LIST_FILTER_H
