#include "path_item_delegate.h"

extern "C" void path_item_delegate(QObject *parent, const int column) {
    PathItemDelegate* delegate = new PathItemDelegate(parent);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

PathItemDelegate::PathItemDelegate(QObject *parent): QStyledItemDelegate(parent) {}

// Function called when the combo it's created. It just put the values into the combo and returns it.
QWidget* PathItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {

    QFileDialog* dialog = new QFileDialog(parent);
    dialog->setFileMode(QFileDialog::FileMode::ExistingFile);
    dialog->setNameFilter("Executable (*.exe)");

    return dialog;
}

void PathItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    QString value = index.model()->data(index, Qt::EditRole).toString();
    QFileDialog* dialog = static_cast<QFileDialog*>(editor);
    dialog->setDirectory(value);
    dialog->show();
}

void PathItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {
    QFileDialog* dialog = static_cast<QFileDialog*>(editor);
    QStringList paths = dialog->selectedFiles();

    if (paths.count() > 0) {
        QString value = paths.value(0);
        model->setData(index, value, Qt::EditRole);
    }
}

void PathItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const {
    editor->setGeometry(option.rect);
}
