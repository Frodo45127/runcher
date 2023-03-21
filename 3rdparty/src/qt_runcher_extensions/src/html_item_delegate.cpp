#include "html_item_delegate.h"
#include <QAbstractItemView>
#include <QAbstractTextDocumentLayout>
#include <QPainter>
#include <QTextDocument>
#include <QTreeView>

extern "C" void html_item_delegate(QObject *parent, const int column) {
    HtmlItemDelegate* delegate = new HtmlItemDelegate(parent);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of HtmlItemDelegate. We use it to store the integer type of the value in the delegate.
HtmlItemDelegate::HtmlItemDelegate(QObject *parent): QStyledItemDelegate(parent) {}

// Function for the delegate to showup properly.
void HtmlItemDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const {
    QStyleOptionViewItem opt(option);
    QTreeView* view = dynamic_cast<QTreeView*>(parent());

    // Remove indentation for category items.
    if (index.column() == 0 && index.data(40).toBool()) {
        opt.rect.adjust(-view->indentation(), 0, 0, 0);
    }

    initStyleOption(&opt, index);

    painter->save();

    QTextDocument doc;
    doc.setHtml(opt.text);

    opt.text = "";
    opt.widget->style()->drawControl(QStyle::CE_ItemViewItem, &opt, painter);
    opt.rect.adjust(view->indentation(), 0, 0, 0);

    painter->translate(opt.rect.left(), opt.rect.top());
    QRect clip(0, 0, opt.rect.width(), opt.rect.height());
    doc.drawContents(painter, clip);

    painter->restore();
}

QSize HtmlItemDelegate::sizeHint(const QStyleOptionViewItem &option, const QModelIndex &index ) const
{
    QStyleOptionViewItem opt = option;
    initStyleOption(&opt, index);

    QTextDocument doc;
    doc.setHtml(opt.text);
    //doc.setTextWidth(opt.rect.width());

    QTreeView* view = dynamic_cast<QTreeView*>(parent());
    return QSize(doc.idealWidth() + (view->indentation()), doc.size().height());
}