use five_gtk::model::color::Rgb;
use five_gtk::model::wall_palette::ensure_contrast;
use five_gtk::model::wall_seed::parse_seed;

const JSON: &str = r##"{"base":"#1E2C2E","text":"#959F9B"}"##;
const CSS: &str = ":root { --color1: #373E41; --color4: #3C6344; --color14: #718969; --color15: #959F9B; --color99: #ffffff; }";

#[test]
fn hex_round_trip() {
    assert_eq!(Rgb::from_hex("#1E2C2E").map(Rgb::to_hex), Some("#1e2c2e".to_string()));
    assert_eq!(Rgb::from_hex("zzz"), None);
}

#[test]
fn hsl_round_trip_keeps_color() {
    let color = Rgb::from_hex("#3c6344").unwrap_or(Rgb { r: 0.0, g: 0.0, b: 0.0 });
    assert_eq!(color.to_hsl().to_rgb().to_hex(), "#3c6344");
}

#[test]
fn seed_reads_palette_colors_one_to_fourteen_only() {
    let seed = parse_seed(JSON, CSS);
    assert_eq!(seed.map(|s| s.candidates.len()), Some(3));
}

#[test]
fn seed_is_none_without_base_color() {
    assert!(parse_seed(r##"{"text":"#ffffff"}"##, CSS).is_none());
}

#[test]
fn contrast_is_raised_to_the_minimum() {
    let bg = Rgb::from_hex("#1e2c2e").unwrap_or(Rgb { r: 0.0, g: 0.0, b: 0.0 });
    let weak = Rgb::from_hex("#2a3a3c").unwrap_or(bg);
    assert!(ensure_contrast(weak, bg, 4.5).contrast(bg) >= 4.5);
}
