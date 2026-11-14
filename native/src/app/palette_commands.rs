//! Commandes proposées par la palette : icône, libellé, raccourci affiché, action.

use std::rc::Rc;

use super::shared::App;

type Action = Rc<dyn Fn(&Rc<App>)>;

/// Commandes proposées : icône, libellé, raccourci affiché, action.
pub fn commands(app: &Rc<App>) -> Vec<(String, String, Option<&'static str>, Action)> {
    let act = |f: fn(&Rc<App>)| -> Action { Rc::new(f) };
    let mut list: Vec<(String, String, Option<&'static str>, Action)> = vec![
        ("plus".into(), "Nouveau dossier".into(), Some("c"), act(|a| a.create_folder())),
        ("up".into(), "Dossier parent".into(), Some("h"), act(|a| a.go_up())),
        ("paste".into(), "Coller".into(), Some("Ctrl V"), act(|a| a.paste_clip())),
        ("pen".into(), "Renommer la sélection".into(), Some("F2"), act(|a| a.rename_items(a.picked()))),
        ("gear".into(), "Ouvrir les paramètres".into(), Some(","), act(|a| a.toggle_settings())),
        ("sun".into(), "Basculer le thème clair / sombre".into(), None, act(|a| a.toggle_theme())),
        ("eye".into(), "Afficher ou masquer les fichiers cachés".into(), Some("."), act(|a| a.toggle_hidden())),
        ("list".into(), "Affichage en liste".into(), None, act(|a| a.set_view("list"))),
        ("grid".into(), "Affichage en grille".into(), None, act(|a| a.set_view("grid"))),
        ("archive".into(), "Analyser l'occupation de ce dossier".into(), None, act(|a| a.analyse_usage())),
        ("copy".into(), "Chercher les doublons dans ce dossier".into(), None, act(|a| a.find_duplicates())),
        ("home".into(), "Aller à : Dossier personnel".into(), None, act(|a| a.go(""))),
        ("recent".into(), "Aller à : Récents".into(), None, act(|a| a.go_view("recent"))),
        ("favorites".into(), "Aller à : Favoris".into(), None, act(|a| a.go_view("favorites"))),
        ("network".into(), "Aller à : Réseau".into(), None, act(|a| a.go_view("network"))),
    ];
    for place in app.st.borrow().places.clone() {
        let path = place.path.clone();
        list.push((place.id.clone(), format!("Aller à : {}", place.label), None, Rc::new(move |a: &Rc<App>| a.go(&path))));
    }
    list
}
