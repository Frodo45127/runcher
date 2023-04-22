#ifndef FLAGS_ITEM_DELEGATE_H
#define FLAGS_ITEM_DELEGATE_H

#include <QStyledItemDelegate>

extern "C" void flags_item_delegate(QObject *parent = nullptr, const int column = 0);

class FlagsItemDelegate: public QStyledItemDelegate {
    Q_OBJECT
public:
    explicit FlagsItemDelegate(QObject *parent = nullptr);
    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void paintIcon(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index, const QString iconId, int &iconWidth, int &pos_x, int margin) const;

private:
    QIcon* icon1;
};

#endif // FLAGS_ITEM_DELEGATE_H
