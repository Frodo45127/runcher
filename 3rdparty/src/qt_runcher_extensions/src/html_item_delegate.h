#ifndef HTML_ITEM_DELEGATE_H
#define HTML_ITEM_DELEGATE_H

#include <QStyledItemDelegate>

extern "C" void html_item_delegate(QObject *parent = nullptr, const int column = 0);

class HtmlItemDelegate: public QStyledItemDelegate {
    Q_OBJECT
public:
    explicit HtmlItemDelegate(QObject *parent = nullptr);
    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    QSize sizeHint(const QStyleOptionViewItem &option, const QModelIndex &index ) const;
};

#endif // HTML_ITEM_DELEGATE_H
