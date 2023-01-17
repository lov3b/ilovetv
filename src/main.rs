use std::io::{self, stdout, Write};
use std::process::Command;
use std::rc::Rc;

use iptvnator_rs::{setup, M3u8, Parser};
fn main() {
    println!("Welcome to iptvnator_rs, the somewhat faster version and certainly harder to write iptvnator");
    let p = Parser::new("iptv.m3u8".to_owned(), setup(), "watched.txt".to_owned());

    let stdin = io::stdin();
    let mut stdout = stdout().lock();
    let mut search_result: Option<Rc<Vec<&M3u8>>> = None;

    loop {
        let mut buf = String::new();

        // Dont't perform a search if user has just watched, instead present the previous search
        if search_result.is_none() {
            print!("Search by name: ");
            stdout.flush().unwrap();
            stdin.read_line(&mut buf).unwrap();
            buf = buf.trim().to_owned();

            // If they want to quit, let them-
            if buf.trim() == "q" {
                break;
            }

            search_result = Some(Rc::new(p.find(&buf)));

            if search_result.as_ref().unwrap().len() == 0 {
                println!("Nothing found");
                stdout.flush().unwrap();
                continue;
            }
        }

        // Let them choose which one to stream
        for (idx, m3u8_item) in search_result.as_ref().unwrap().iter().enumerate().rev() {
            println!("  {}: {}", idx + 1, m3u8_item);
        }
        print!("Which one do you wish to stream? [q | s]: ");
        stdout.flush().unwrap();
        buf = String::new();
        stdin.read_line(&mut buf).unwrap();

        // If they want to quit, let them-
        if buf.trim() == "q" {
            break;
        }

        let choosen = buf.trim().parse::<usize>();
        match choosen {
            Ok(k) => {
                let search_result = search_result.as_ref().unwrap().clone();
                stream(&(search_result[k - 1]))
            }
            Err(e) => println!("Have to be a valid number! {:?}", e),
        }
    }

    p.save_watched();
}

fn stream(m3u8item: &M3u8) {
    // Well I know that this is frowned upon, but it's honestly the most efficient way of doing this
    let ptr = m3u8item as *const M3u8;
    let ptr = ptr as *mut M3u8;
    let mut item = unsafe { &mut *ptr };
    item.watched = true;

    Command::new("mpv")
        .arg(&m3u8item.link)
        .output()
        .expect("Could not listen for output");
}
