mod m3u8;
mod parser;
use std::{
    fs,
    io::{stdin, stdout, Write},
    process,
};

pub use m3u8::M3u8;
pub use parser::Parser;
mod config;

use directories;

#[test]
pub fn aaaaaaaa() {
    let a = directories::ProjectDirs::from("com", "billenius", "iptvnator_rs").unwrap();
    let datadir = a.data_dir();
    let configdir = a.config_dir();
    println!("{:?}", datadir);
    println!("{:?}", configdir);
}

pub fn setup() -> String {
    let project_dirs = directories::ProjectDirs::from("com", "billenius", "iptvnator_rs").unwrap();
    let config_dir = project_dirs.config_dir();
    let ilovetv_config_file = config_dir.join("iptv_url.txt");
    if ilovetv_config_file.exists() {
        return fs::read_to_string(&ilovetv_config_file).expect("Failed to read iptv_url");
    }

    println!("Hello, I would need an url to your iptv/m3u/m3u8 stream");
    print!("enter url: ");
    let mut stdout = stdout().lock();
    stdout.flush().unwrap();
    let mut url = String::new();
    let stdin = stdin();
    let _ = stdin.read_line(&mut url);
    print!("Are you sure? (Y/n) ");
    stdout.flush().unwrap();
    let mut yn = String::new();
    let _ = stdin.read_line(&mut yn);
    if yn.trim() == "n" {
        setup();
    }

    let _ = fs::create_dir_all(config_dir);
    if let Err(e) = fs::write(ilovetv_config_file, url.trim()) {
        eprintln!("{:?}", e);
        process::exit(-1);
    }

    url.to_string()
}
