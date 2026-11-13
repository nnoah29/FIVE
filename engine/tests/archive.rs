mod support;

use std::fs;
use std::io::Write;
use std::process::Command;

use five_engine::archive::list_archive;
use support::Sandbox;

fn stored_zip(entries: &[(&str, &str)]) -> Vec<u8> {
    let (mut body, mut directory) = (Vec::new(), Vec::new());
    for (name, data) in entries {
        let offset = body.len() as u32;
        let (size, len) = (data.len() as u32, name.len() as u16);
        body.extend([0x50, 0x4b, 0x03, 0x04, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        body.extend(size.to_le_bytes().iter().chain(size.to_le_bytes().iter()));
        body.extend(len.to_le_bytes().iter().chain([0u8, 0].iter()));
        body.extend(name.bytes().chain(data.bytes()));
        directory.extend([0x50, 0x4b, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        directory.extend(size.to_le_bytes().iter().chain(size.to_le_bytes().iter()));
        directory.extend(len.to_le_bytes().iter().chain([0u8; 12].iter()));
        directory.extend(offset.to_le_bytes().iter().chain(name.bytes().collect::<Vec<_>>().iter()));
    }
    let (count, size, start) = (entries.len() as u16, directory.len() as u32, body.len() as u32);
    body.extend(directory);
    body.extend([0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
    body.extend(count.to_le_bytes().iter().chain(count.to_le_bytes().iter()));
    body.extend(size.to_le_bytes().iter().chain(start.to_le_bytes().iter()));
    body.extend([0u8, 0]);
    body
}

#[test]
fn zip_members_are_listed_with_sizes_and_folders() {
    let sandbox = Sandbox::new();
    let path = sandbox.dir.join("a.zip");
    fs::File::create(&path).unwrap().write_all(&stored_zip(&[("dossier/", ""), ("dossier/n.txt", "hello")])).unwrap();
    let listing = list_archive(&path).unwrap();
    assert_eq!(listing["items"][0]["d"], true);
    assert_eq!(listing["items"][1]["n"], "dossier/n.txt");
    assert_eq!(listing["items"][1]["s"], 5);
    assert_eq!(listing["more"], false);
}

#[test]
fn tar_and_tar_gz_members_are_listed() {
    let sandbox = Sandbox::new();
    sandbox.write("src/a.txt", "12345");
    sandbox.write("src/sous/b.txt", "1");
    for (flags, name) in [("-cf", "a.tar"), ("-czf", "a.tgz")] {
        let status = Command::new("tar").args([flags, name, "src"]).current_dir(&sandbox.dir).status().unwrap();
        assert!(status.success());
        let listing = list_archive(&sandbox.dir.join(name)).unwrap();
        let names: Vec<&str> = listing["items"].as_array().unwrap().iter().map(|i| i["n"].as_str().unwrap()).collect();
        assert!(names.contains(&"src/a.txt") && names.contains(&"src/sous/b.txt"), "{name}: {names:?}");
    }
}

#[test]
fn unreadable_files_are_refused() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("faux.zip", "ce n'est pas une archive");
    assert_eq!(list_archive(&path).unwrap_err().code, 400);
}

#[test]
fn listing_stops_at_500_entries() {
    let sandbox = Sandbox::new();
    for n in 0..520 {
        sandbox.write(&format!("src/f{n:03}"), "x");
    }
    let status = Command::new("tar").args(["-cf", "g.tar", "src"]).current_dir(&sandbox.dir).status().unwrap();
    assert!(status.success());
    let listing = list_archive(&sandbox.dir.join("g.tar")).unwrap();
    assert_eq!(listing["items"].as_array().unwrap().len(), 500);
    assert_eq!(listing["more"], true);
}
