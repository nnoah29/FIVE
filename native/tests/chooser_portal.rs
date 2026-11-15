use five_gtk::chooser::portal_xml::{FILE_CHOOSER_XML, REQUEST_XML};
use gtk4::gio::DBusNodeInfo;

#[test]
fn the_exported_interfaces_are_valid_dbus_descriptions() {
    let chooser = DBusNodeInfo::for_xml(FILE_CHOOSER_XML).unwrap();
    let info = chooser.lookup_interface("org.freedesktop.impl.portal.FileChooser").unwrap();
    for method in ["OpenFile", "SaveFile", "SaveFiles"] {
        assert!(info.lookup_method(method).is_some(), "{method}");
    }
    let request = DBusNodeInfo::for_xml(REQUEST_XML).unwrap();
    assert!(request.lookup_interface("org.freedesktop.impl.portal.Request").unwrap().lookup_method("Close").is_some());
}
