#include "flags_item_delegate.h"
#include <QAbstractItemView>
#include <QAbstractTextDocumentLayout>
#include <QDebug>
#include <QPainter>
#include <QPixmapCache>
#include <QTextDocument>
#include <QTreeView>

const int FLAG_MOD_IS_OUTDATED = 31;
const int FLAG_MOD_DATA_IS_OLDER_THAN_SECONDARY = 32;
const int FLAG_MOD_DATA_IS_OLDER_THAN_CONTENT = 33;
const int FLAG_MOD_SECONDARY_IS_OLDER_THAN_CONTENT = 34;

extern "C" void flags_item_delegate(QObject *parent, const int column) {
    FlagsItemDelegate* delegate = new FlagsItemDelegate(parent);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of FlagsItemDelegate.
FlagsItemDelegate::FlagsItemDelegate(QObject *parent): QStyledItemDelegate(parent) {
    icon1 = new QIcon();
}

// Function for the delegate to showup properly.
void FlagsItemDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const {
    QStyledItemDelegate::paint(painter, option, index);

    int iconsToShow = 0;
    int pos_x = 4;
    painter->save();

    if (index.data(FLAG_MOD_IS_OUTDATED).toBool()) {
        iconsToShow += 1;
    }

    int iconWidth = iconsToShow > 0 ? ((option.rect.width() / iconsToShow) - 4) : 16;
    iconWidth = std::min(16, iconWidth);
    const int margin = (option.rect.height() - iconWidth) / 2;
    painter->translate(option.rect.topLeft());

    if (index.data(FLAG_MOD_IS_OUTDATED).toBool()) {
        paintIcon(painter, option, index, "outdated.png", iconWidth, pos_x, margin);
        pos_x += 3;
    }

    if (index.data(FLAG_MOD_DATA_IS_OLDER_THAN_SECONDARY).toBool()) {
        paintIcon(painter, option, index, "data_older_than_secondary.png", iconWidth, pos_x, margin);
        pos_x += 3;
    }

    if (index.data(FLAG_MOD_DATA_IS_OLDER_THAN_CONTENT).toBool()) {
        paintIcon(painter, option, index, "data_older_than_content.png", iconWidth, pos_x, margin);
        pos_x += 3;
    }

    if (index.data(FLAG_MOD_SECONDARY_IS_OLDER_THAN_CONTENT).toBool()) {
        paintIcon(painter, option, index, "secondary_older_than_content.png", iconWidth, pos_x, margin);
        pos_x += 3;
    }

    painter->restore();
}

void FlagsItemDelegate::paintIcon(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index, const QString iconId, int &iconWidth, int &pos_x, int margin) const {

    if (iconId.isEmpty()) {
        pos_x += iconWidth + 4;
        return;
    }

    QPixmap icon;
    QString fullIconId = QString("%1_%2").arg(iconId).arg(iconWidth);

    if (!QPixmapCache::find(fullIconId, &icon)) {
        icon = QIcon(QString("./icons/%1").arg(iconId)).pixmap(iconWidth, iconWidth);

        if (icon.isNull()) {
            qWarning() << "Failed to load icon file id: " << iconId;
            icon = QIcon::fromTheme(iconId).pixmap(iconWidth, iconWidth);
        }

        // If we find them in files, make sure to resize them.
        else {
            icon.scaled(QSize(12, 12), Qt::KeepAspectRatio);
        }

        if (icon.isNull()) {
            qWarning() << "Failed to load icon from theme with id: " << iconId;
        }

        QPixmapCache::insert(fullIconId, icon);
    }

    painter->drawPixmap(pos_x, margin, iconWidth, iconWidth, icon);
    pos_x += iconWidth + 4;
}
