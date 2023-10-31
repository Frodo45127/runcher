#ifndef PACK_LIST_TREE_VIEW_H
#define PACK_LIST_TREE_VIEW_H

#include <QTreeView>
#include <QDropEvent>

extern "C" QTreeView* new_pack_list_tree_view(QWidget *parent = nullptr);

class PackListTreeView : public QTreeView {
    Q_OBJECT
signals:
    void itemDrop(QModelIndex const &,int);
public:
    explicit PackListTreeView(QWidget *parent = nullptr);

protected:
    void dragEnterEvent(QDragEnterEvent *event) override;
    void dragMoveEvent(QDragMoveEvent *event) override;
    void dragLeaveEvent(QDragLeaveEvent *event) override;
    void dropEvent(QDropEvent *event) override;
};

#endif // PACK_LIST_TREE_VIEW_H
