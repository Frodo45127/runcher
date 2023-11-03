#ifndef GAME_SELECTOR_ITEM_DELEGATE_H
#define GAME_SELECTOR_ITEM_DELEGATE_H

#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QAbstractItemView>
#include <QMenu>

extern "C" void game_selector_item_delegate(QObject *parent, const int column, const QStringList* game_keys);

class GameSelectorItemDelegate : public QStyledItemDelegate
{
    Q_OBJECT

public:

    explicit GameSelectorItemDelegate(QObject *parent, const QStringList* game_keys);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

private:
    const QStringList* keys;
};
#endif // GAME_SELECTOR_ITEM_DELEGATE_H
