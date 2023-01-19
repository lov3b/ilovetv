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
mod downloader;
use directories::ProjectDirs;
pub use downloader::download_with_progress;

pub fn setup() -> String {
    let project_dirs = ProjectDirs::from("com", "billenius", "iptvnator_rs").unwrap();
    let config_dir = project_dirs.config_dir();
    let ilovetv_config_file = config_dir.join("iptv_url.txt");
    if ilovetv_config_file.exists() {
        return fs::read_to_string(&ilovetv_config_file).expect("Failed to read iptv_url");
    }

    let mut stdout = stdout().lock();
    let stdin = stdin();

    println!("Hello, I would need an url to your iptv/m3u/m3u8 stream");
    let url = loop {
        print!("enter url: ");
        stdout.flush().unwrap();
        let mut url = String::new();
        let _ = stdin.read_line(&mut url);

        print!("Are you sure? (Y/n) ");
        stdout.flush().unwrap();
        let mut yn = String::new();
        let _ = stdin.read_line(&mut yn);

        if yn.to_lowercase() != "n" {
            break url.trim().to_string();
        }
    };

    let _ = fs::create_dir_all(config_dir);
    if let Err(e) = fs::write(ilovetv_config_file, &url) {
        eprintln!("{:?}", e);
        process::exit(-1);
    }

    url
}
