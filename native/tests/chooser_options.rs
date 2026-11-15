use five_gtk::chooser::options::{bytes_of, open_mode, parse_request, path_from_bytes};
use five_gtk::chooser::outcome::{Choice, results_variant};
use five_gtk::chooser::request::{Mode, Request};
use gtk4::glib::{Variant, VariantDict};
use gtk4::prelude::*;

fn options(build: impl Fn(&VariantDict)) -> Variant {
    let dict = VariantDict::new(None);
    build(&dict);
    dict.end()
}

#[test]
fn portal_options_become_a_request() {
    let dir = std::env::temp_dir();
    let filters = vec![("Images".to_string(), vec![(1u32, "image/*".to_string())]), ("Texte".to_string(), vec![(0u32, "*.txt".to_string())])];
    let opts = options(|d| {
        d.insert_value("multiple", &true.to_variant());
        d.insert_value("accept_label", &"Choisir".to_variant());
        d.insert_value("filters", &filters.to_variant());
        d.insert_value("current_filter", &filters[1].to_variant());
        d.insert_value("current_folder", &bytes_of(&dir));
    });
    let request = parse_request(Mode::Open, "Ouvrir", &opts);
    assert!(request.multiple && request.start_dir == dir);
    assert_eq!((request.accept_label.as_deref(), request.filters.len(), request.current_filter), (Some("Choisir"), 2, 1));
}

#[test]
fn directory_option_switches_to_folder_mode_and_bytes_drop_the_nul() {
    assert_eq!(open_mode(&options(|d| d.insert_value("directory", &true.to_variant()))), Mode::Folder);
    assert_eq!(open_mode(&options(|_| {})), Mode::Open);
    assert_eq!(path_from_bytes(b"/tmp/x\0"), std::path::PathBuf::from("/tmp/x"));
}

#[test]
fn results_carry_file_uris_and_the_chosen_filter() {
    let mut request = Request::new(Mode::Open, "t");
    request.filters = vec![five_gtk::chooser::request::Filter { name: "Tout".into(), rules: vec![] }];
    let choice = Choice { paths: vec!["/tmp/a b.txt".into()], filter: Some(0) };
    let dict = VariantDict::new(Some(&results_variant(&request, &choice)));
    assert_eq!(dict.lookup::<Vec<String>>("uris").unwrap().unwrap(), vec!["file:///tmp/a%20b.txt"]);
    assert!(dict.lookup_value("current_filter", None).is_some());
}

#[test]
fn text_format_options_from_a_command_line_client_are_understood() {
    let opts = Variant::parse(None, "{'current_folder': <b'/tmp'>, 'current_name': <'a.txt'>}").unwrap();
    let request = parse_request(Mode::Save, "t", &opts);
    assert_eq!((request.start_dir, request.current_name.as_str()), (std::path::PathBuf::from("/tmp"), "a.txt"));
}
