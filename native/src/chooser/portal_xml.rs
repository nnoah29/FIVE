//! Description D-Bus du service : interface FileChooser du portail et interface Request.

/// Interface `org.freedesktop.impl.portal.FileChooser` : ouvrir, enregistrer, enregistrer plusieurs fichiers.
pub const FILE_CHOOSER_XML: &str = r#"<node>
<interface name="org.freedesktop.impl.portal.FileChooser">
<method name="OpenFile">
<arg type="o" name="handle" direction="in"/><arg type="s" name="app_id" direction="in"/>
<arg type="s" name="parent_window" direction="in"/><arg type="s" name="title" direction="in"/>
<arg type="a{sv}" name="options" direction="in"/>
<arg type="u" name="response" direction="out"/><arg type="a{sv}" name="results" direction="out"/>
</method>
<method name="SaveFile">
<arg type="o" name="handle" direction="in"/><arg type="s" name="app_id" direction="in"/>
<arg type="s" name="parent_window" direction="in"/><arg type="s" name="title" direction="in"/>
<arg type="a{sv}" name="options" direction="in"/>
<arg type="u" name="response" direction="out"/><arg type="a{sv}" name="results" direction="out"/>
</method>
<method name="SaveFiles">
<arg type="o" name="handle" direction="in"/><arg type="s" name="app_id" direction="in"/>
<arg type="s" name="parent_window" direction="in"/><arg type="s" name="title" direction="in"/>
<arg type="a{sv}" name="options" direction="in"/>
<arg type="u" name="response" direction="out"/><arg type="a{sv}" name="results" direction="out"/>
</method>
</interface>
</node>"#;

/// Interface `org.freedesktop.impl.portal.Request` : le portail appelle `Close` pour annuler une demande.
pub const REQUEST_XML: &str = r#"<node><interface name="org.freedesktop.impl.portal.Request"><method name="Close"/></interface></node>"#;
