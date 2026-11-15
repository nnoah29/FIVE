mod support_dir;

use std::fs;

use five_gtk::chooser::preview_parts::text_excerpt;

#[test]
fn a_text_file_gives_its_first_lines() {
    let dir = support_dir::sample();
    let long: String = (0..100).map(|i| format!("ligne {i}\n")).collect();
    fs::write(dir.join("long.txt"), long).unwrap();
    let excerpt = text_excerpt(&dir.join("long.txt")).unwrap();
    assert_eq!((excerpt.lines().count(), excerpt.lines().next()), (40, Some("ligne 0")));
}

#[test]
fn binary_and_empty_files_have_no_text_preview() {
    let dir = support_dir::sample();
    fs::write(dir.join("data.bin"), [0u8, 159, 146, 150, 0, 1]).unwrap();
    fs::write(dir.join("vide.txt"), "  \n").unwrap();
    assert!(text_excerpt(&dir.join("data.bin")).is_none());
    assert!(text_excerpt(&dir.join("vide.txt")).is_none());
}

#[test]
fn a_cut_in_the_middle_of_a_character_is_tolerated() {
    let dir = support_dir::sample();
    fs::write(dir.join("accents.txt"), "é".repeat(3000)).unwrap();
    assert!(text_excerpt(&dir.join("accents.txt")).is_some());
}
