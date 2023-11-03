#include "game_selector_item_delegate.h"
#include <QWidgetAction>
#include <QCheckBox>
#include <QGridLayout>

extern "C" void game_selector_item_delegate(QObject *parent, const int column, const QStringList* game_keys) {
    GameSelectorItemDelegate* delegate = new GameSelectorItemDelegate(parent, game_keys);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

GameSelectorItemDelegate::GameSelectorItemDelegate(QObject *parent, const QStringList* game_keys): QStyledItemDelegate(parent) {
    keys = game_keys;
}

// Function called when the combo it's created. It just put the values into the combo and returns it.
QWidget* GameSelectorItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {
    QMenu* menu = new QMenu(parent);

    for(int i = 0; i < keys->count(); i++) {
        QWidgetAction* action = new QWidgetAction(menu);
        QWidget* widget = new QWidget(menu);
        QCheckBox* check = new QCheckBox(widget);
        QGridLayout* layout = new QGridLayout(widget);
        action->setDefaultWidget(widget);
        check->setText(keys->value(i));

        layout->addWidget(check);
        widget->setLayout(layout);

        menu->addAction(action);
    }

    return menu;
}

void GameSelectorItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    QMenu* menu = static_cast<QMenu*>(editor);

    QString value = index.model()->data(index, Qt::EditRole).toString();
    QStringList value_split = value.split(",");

    for(int i = 0; i < keys->count(); i++) {
        if (value_split.contains(keys->value(i))) {
            QWidgetAction* action = static_cast<QWidgetAction*>(menu->actions().value(i));
            QWidget* widget = action->defaultWidget();
            QGridLayout* layout = static_cast<QGridLayout*>(widget->layout());
            QCheckBox* check = static_cast<QCheckBox*>(layout->itemAt(0)->widget());
            check->setChecked(true);
        }
    }

    menu->move(QCursor::pos());
}

void GameSelectorItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {
    QMenu* menu = static_cast<QMenu*>(editor);

    QStringList* value_split = new QStringList();

    for(int i = 0; i < keys->count(); i++) {
        QWidgetAction* action = static_cast<QWidgetAction*>(menu->actions().value(i));
        QWidget* widget = action->defaultWidget();
        QGridLayout* layout = static_cast<QGridLayout*>(widget->layout());
        QCheckBox* check = static_cast<QCheckBox*>(layout->itemAt(0)->widget());
        if (check->isChecked()) {
            value_split->append(check->text());
        }
    }

    QString value = value_split->join(",");
    model->setData(index, value, Qt::EditRole);

    menu->close();
}

void GameSelectorItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const {
    //editor->setGeometry(option.rect);
}
