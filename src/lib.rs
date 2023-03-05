mod config;
mod downloader;
pub mod getm3u8;
mod grandmother;
mod m3u8;
mod opt;
mod parser;
mod playlist;

use std::io::{stdin, stdout, Stdin, StdoutLock, Write};

use async_recursion::async_recursion;
pub use config::Configuration;
pub use downloader::download_with_progress;
pub use getm3u8::{GetM3u8, GetPlayPath, WatchedFind};
pub use grandmother::GrandMother;
pub use m3u8::{M3u8, OfflineEntry};
pub use opt::{Mode, Opt};
pub use parser::{OfflineParser, Parser};
pub use playlist::Playlist;

pub const JSON_CONFIG_FILENAME: &'static str = "config.json";
pub const APP_IDENTIFIER: [&'static str; 3] = ["com", "billenius", "ilovetv"];
pub const STANDARD_PLAYLIST_FILENAME: &'static str = "playlist.m3u8";
pub const STANDARD_SEEN_LINKS_FILENAME: &'static str = "watched_links.json";
pub const STANDARD_OFFLINE_FILENAME: &'static str = "ilovetv_offline.json";
pub const MAX_TRIES: u8 = 4;

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

#[async_recursion(?Send)]
pub async fn get_gm(
    mode: Mode,
    readline: &mut Readline<'_>,
    config: Configuration,
) -> Result<GrandMother, String> {
    match mode {
        Mode::Online => GrandMother::new(config).await,
        Mode::Offline => Ok(GrandMother::new_offline(config)),
        Mode::Ask => loop {
            let input = readline
                .input("Online/Offline mode? [1/2] ")
                .trim()
                .parse::<u8>();
            if let Ok(num) = input {
                if num == 1 {
                    return get_gm(Mode::Online, readline, config).await;
                } else if num == 2 {
                    return get_gm(Mode::Offline, readline, config).await;
                }
                println!("Has to be either 1 (Onine) or 2 (Offline)");
            } else {
                println!("Has to be a number");
            }
        },
    }
}
