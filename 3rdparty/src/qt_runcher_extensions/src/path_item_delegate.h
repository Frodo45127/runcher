#ifndef PATH_ITEM_DELEGATE_H
#define PATH_ITEM_DELEGATE_H

#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QAbstractItemView>
#include <QFileDialog>

extern "C" void path_item_delegate(QObject *parent = nullptr, const int column = 0);

class PathItemDelegate : public QStyledItemDelegate
{
    Q_OBJECT

public:

    explicit PathItemDelegate(QObject *parent = nullptr);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

private:

};
#endif // PATH_ITEM_DELEGATE_H
