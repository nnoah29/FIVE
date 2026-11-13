mod support;

use five_engine::disks::mounted_disks;
use five_engine::urlcodec::{decode, encode_path};
use support::Sandbox;

#[test]
fn url_codec_round_trips_unicode_and_spaces() {
    assert_eq!(encode_path("/home/a b/é.txt"), "/home/a%20b/%C3%A9.txt");
    assert_eq!(decode("/home/a%20b/%C3%A9.txt"), "/home/a b/é.txt");
    assert_eq!(decode("100%"), "100%");
}

#[test]
fn mounted_disks_report_sizes_for_real_devices() {
    let sandbox = Sandbox::new();
    for disk in mounted_disks(&sandbox.root()) {
        assert!(disk["dev"].as_str().unwrap().starts_with("/dev/"));
        assert!(disk["t"].as_u64().unwrap() >= disk["f"].as_u64().unwrap());
    }
}
