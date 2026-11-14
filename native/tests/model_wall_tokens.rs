use five_gtk::model::color::Rgb;
use five_gtk::model::wall_palette::wall_tokens;
use five_gtk::model::wall_seed::parse_seed;

const JSON: &str = r##"{"base":"#1E2C2E","text":"#959F9B"}"##;
const CSS: &str = "";

fn token(tokens: &[(&'static str, String)], name: &str) -> String {
    tokens.iter().find(|(n, _)| *n == name).map(|(_, v)| v.clone()).unwrap_or_default()
}

#[test]
fn dark_wallpaper_gives_dark_palette_with_readable_text() {
    let seed = parse_seed(JSON, CSS).expect("seed");
    let (tokens, dark) = wall_tokens(&seed);
    assert!(dark);
    assert_eq!(token(&tokens, "bg"), "#1e2c2e");
    let (text, bg) = (Rgb::from_hex(&token(&tokens, "text")), Rgb::from_hex(&token(&tokens, "bg")));
    assert!(text.zip(bg).is_some_and(|(t, b)| t.contrast(b) >= 10.0));
}

#[test]
fn gray_wallpaper_uses_text_color_as_accent() {
    let seed = parse_seed(r##"{"base":"#202020","text":"#d0d0d0"}"##, "").expect("seed");
    let (tokens, _) = wall_tokens(&seed);
    assert_eq!(token(&tokens, "accent"), token(&tokens, "text"));
}
