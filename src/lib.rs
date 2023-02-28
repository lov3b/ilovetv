mod m3u8;
mod parser;
use std::{
    fs,
    io::{stdin, stdout, Stdin, StdoutLock, Write},
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

    let mut readline = Readline::new();

    println!("Hello, I would need an url to your iptv/m3u/m3u8 stream");
    let url = loop {
        let url = readline.input("enter url: ");
        let yn = readline.input("Are you sure? (Y/n) ");

        if yn.trim().to_lowercase() != "n" {
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

pub struct Readline<'a> {
    stdout: StdoutLock<'a>,
    stdin: Stdin,
}

impl<'a> Readline<'a> {
    pub fn new() -> Self {
        Self {
            stdout: stdout().lock(),
            stdin: stdin(),
        }
    }

    pub fn input(&mut self, toprint: &str) -> String {
        print!("{}", toprint);
        self.stdout.flush().unwrap();
        let mut buffer = String::new();
        self.stdin.read_line(&mut buffer).unwrap();
        buffer
    }
}

/**
 * I know that this isn't considered true rusty code, but the places it's used in is
 * safe. For this I'll leave the funciton as unsafe, so to better see it's uses. 
 * This solution makes the uses BLAZINGLY FAST which moreover is the most rusty you can get.
 */
pub unsafe fn get_mut_ref<T>(reference: &T) -> &mut T {
    let ptr = reference as *const T as *mut T;
    &mut *ptr
}
