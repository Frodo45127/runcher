#ifndef PACK_LIST_MODEL_H
#define PACK_LIST_MODEL_H

#include <QStandardItemModel>
#include <QStringListModel>
#include <QDropEvent>
#include <QDebug>
#include <QMimeData>

extern "C" QStandardItemModel* new_pack_list_model();

class PackListModel : public QStandardItemModel {
    Q_OBJECT
public:
    Qt::ItemFlags flags(const QModelIndex &index) const;
};

#endif // PACK_LIST_MODEL_H
