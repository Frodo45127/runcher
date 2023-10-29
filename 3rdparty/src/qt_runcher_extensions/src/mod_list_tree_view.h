#ifndef MOD_LIST_TREE_VIEW_H
#define MOD_LIST_TREE_VIEW_H

#include <QTreeView>
#include <QDropEvent>

extern "C" QTreeView* new_mod_list_tree_view(QWidget *parent = nullptr);

class ModListTreeView : public QTreeView {
    Q_OBJECT
signals:
    void itemDrop(QModelIndex const &,int);
public:
    explicit ModListTreeView(QWidget *parent = nullptr);

protected:
    void dragEnterEvent(QDragEnterEvent *event) override;
    void dragMoveEvent(QDragMoveEvent *event) override;
    void dragLeaveEvent(QDragLeaveEvent *event) override;
    void dropEvent(QDropEvent *event) override;
};

#endif // MOD_LIST_TREE_VIEW_H
