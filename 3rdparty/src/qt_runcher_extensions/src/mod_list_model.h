#ifndef MOD_LIST_MODEL_H
#define MOD_LIST_MODEL_H

#include <QStandardItemModel>
#include <QStringListModel>
#include <QDropEvent>
#include <QDebug>
#include <QMimeData>

extern "C" QStandardItemModel* new_mod_list_model();

class ModListModel : public QStandardItemModel {
    Q_OBJECT
public:
    Qt::ItemFlags flags(const QModelIndex &index) const;
};

#endif // MOD_LIST_MODEL_H
