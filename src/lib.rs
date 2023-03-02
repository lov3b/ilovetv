mod m3u8;
mod parser;
use std::io::{stdin, stdout, Stdin, StdoutLock, Write};

pub use m3u8::{DataEntry, M3u8};
pub use parser::Parser;
mod config;
pub use config::Configuration;
mod downloader;
pub use downloader::download_with_progress;

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
