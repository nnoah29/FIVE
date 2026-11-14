//! Recherche de widgets dans un arbre, pour le pilotage par script.

use gtk4::prelude::*;

/// Premier descendant du type voulu, en parcourant l'arbre en profondeur.
pub fn find_widget<T: IsA<gtk4::Widget>>(root: &gtk4::Widget) -> Option<T> {
    let mut child = root.first_child();
    while let Some(widget) = child {
        if let Ok(found) = widget.clone().downcast::<T>() {
            return Some(found);
        }
        if let Some(found) = find_widget::<T>(&widget) {
            return Some(found);
        }
        child = widget.next_sibling();
    }
    None
}

/// Premier descendant portant la classe CSS donnée.
pub fn find_class(root: &gtk4::Widget, class: &str) -> Option<gtk4::Widget> {
    let mut child = root.first_child();
    while let Some(widget) = child {
        if widget.has_css_class(class) {
            return Some(widget);
        }
        if let Some(found) = find_class(&widget, class) {
            return Some(found);
        }
        child = widget.next_sibling();
    }
    None
}
