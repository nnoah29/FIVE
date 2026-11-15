use five_gtk::chooser::matching::{filter_accepts, glob_match};
use five_gtk::chooser::request::{Filter, FilterRule};

fn chars(text: &str) -> Vec<char> {
    text.chars().collect()
}

fn filter(rules: &[(u32, &str)]) -> Filter {
    Filter { name: "F".into(), rules: rules.iter().map(|(kind, pattern)| FilterRule { kind: *kind, pattern: pattern.to_string() }).collect() }
}

#[test]
fn globs_support_star_question_mark_and_sets() {
    assert!(glob_match(&chars("*.png"), &chars("photo.PNG")));
    assert!(glob_match(&chars("a?c"), &chars("abc")));
    assert!(glob_match(&chars("*.[pP][nN][gG]"), &chars("x.png")));
    assert!(!glob_match(&chars("*.png"), &chars("photo.jpg")));
    assert!(!glob_match(&chars("a?c"), &chars("ac")));
}

#[test]
fn mime_rules_accept_exact_types_and_families() {
    assert!(filter_accepts(&filter(&[(1, "image/*")]), "a.png"));
    assert!(filter_accepts(&filter(&[(1, "application/pdf")]), "a.pdf"));
    assert!(!filter_accepts(&filter(&[(1, "image/*")]), "a.pdf"));
}

#[test]
fn an_empty_filter_accepts_everything_and_rules_are_alternatives() {
    assert!(filter_accepts(&filter(&[]), "anything.bin"));
    assert!(filter_accepts(&filter(&[(0, "*.txt"), (0, "*.md")]), "notes.md"));
}
