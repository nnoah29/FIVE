//! Lecture des nombres et des lettres de commande d'un chemin SVG.

/// Curseur sur le texte d'un chemin SVG.
pub struct Scanner<'a> {
    text: &'a [u8],
    at: usize,
}

impl<'a> Scanner<'a> {
    /// Démarre la lecture au début du texte.
    pub fn new(text: &'a str) -> Self {
        Self { text: text.as_bytes(), at: 0 }
    }

    /// Saute les espaces et les virgules.
    fn skip_separators(&mut self) {
        while self.text.get(self.at).is_some_and(|b| b.is_ascii_whitespace() || *b == b',') {
            self.at += 1;
        }
    }

    /// Lit une lettre de commande, si la suite en est une.
    pub fn command(&mut self) -> Option<char> {
        self.skip_separators();
        let letter = *self.text.get(self.at)?;
        letter.is_ascii_alphabetic().then(|| {
            self.at += 1;
            letter as char
        })
    }

    /// Indique si un nombre suit.
    pub fn has_number(&mut self) -> bool {
        self.skip_separators();
        self.text.get(self.at).is_some_and(|b| b.is_ascii_digit() || matches!(b, b'-' | b'+' | b'.'))
    }

    /// Lit un nombre : signe, chiffres, partie décimale (`5.9.8` donne 5.9 puis 0.8).
    pub fn number(&mut self) -> Option<f64> {
        self.skip_separators();
        let start = self.at;
        if matches!(self.text.get(self.at), Some(b'-' | b'+')) {
            self.at += 1;
        }
        while self.text.get(self.at).is_some_and(|b| b.is_ascii_digit()) {
            self.at += 1;
        }
        if self.text.get(self.at) == Some(&b'.') {
            self.at += 1;
            while self.text.get(self.at).is_some_and(|b| b.is_ascii_digit()) {
                self.at += 1;
            }
        }
        std::str::from_utf8(&self.text[start..self.at]).ok()?.parse().ok()
    }
}
