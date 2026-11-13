//! Titres de la page : libellé des vues virtuelles et titre du dossier courant.

use crate::app::shared::App;

/// Libellé de chaque vue virtuelle.
pub fn view_label(view: &str) -> &'static str {
    match view {
        "recent" => "Récents",
        "favorites" => "Favoris",
        _ => "Réseau",
    }
}

impl App {
    /// Titre de la page selon la vue, la recherche ou le dossier.
    pub fn page_title(&self) -> String {
        let st = self.st.borrow();
        if let Some(view) = &st.virtual_view {
            return view_label(view).to_string();
        }
        if st.searching() {
            return "Recherche".to_string();
        }
        match st.path.rsplit('/').next() {
            Some(_) if st.path == five_engine::places::TRASH => "Corbeille".to_string(),
            Some(name) if !st.path.is_empty() => name.to_string(),
            _ => "Dossier personnel".to_string(),
        }
    }
}
