use std::io::{self, stdout, Write};
use std::num::ParseIntError;
use std::process::Command;
use std::rc::Rc;

use iptvnator_rs::{download_with_progress, setup, M3u8, Parser};

#[tokio::main]
async fn main() {
    println!("Welcome to iptvnator_rs, the port of my iptvprogram written in python, now in rust BLAZINGLY FAST\n");
    let parser = Parser::new("iptv.m3u8".to_owned(), setup(), "watched.txt".to_owned()).await;

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

            search_result = Some(Rc::new(parser.find(&buf)));

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
        print!("Which one do you wish to stream? [ q/s/r/d ]: ");
        stdout.flush().unwrap();
        buf = String::new();
        stdin.read_line(&mut buf).unwrap();

        let user_wish = buf.trim();
        // If they want to quit, let them-
        if user_wish == "q" {
            break;
        } else if user_wish == "s" {
            search_result = None;
            continue;
        } else if user_wish == "r" {
            println!("Refreshing local m3u8-file");
            search_result = None;

            // I know that this is also frowned upon, but it is perfectly safe right here,
            // even though the borrowchecker complains
            {
                let ptr = &parser as *const Parser as *mut Parser;
                let p = unsafe { &mut *ptr };
                p.forcefully_update().await;
            }
            continue;
        } else if user_wish == "d" {
            print!("Download all or select in comma separated [A]: ");
            stdout.flush().unwrap();

            let mut selection = String::new();
            stdin.read_line(&mut selection).unwrap();

            let selection = selection.trim();
            let to_download = loop {
                break if selection.to_lowercase() == "a" {
                    println!("Downloading all");
                    search_result.as_ref().unwrap().clone()
                } else {
                    let selections = selection
                        .split(",")
                        .map(|x| x.trim().parse::<usize>())
                        .collect::<Vec<Result<usize, ParseIntError>>>();

                    for selection in selections.iter() {
                        if selection.is_err() {
                            println!("Not a valid number");
                            continue;
                        }
                    }
                    let selections = selections.into_iter().map(|x| x.unwrap() - 1);
                    let mut final_selections = Vec::new();
                    for selection in selections {
                        final_selections.push((search_result.as_ref().unwrap())[selection]);
                    }

                    Rc::new(final_selections)
                };
            };
            download_m3u8(to_download).await;
        }

        let choosen = user_wish.parse::<usize>();
        match choosen {
            Ok(k) => {
                let search_result = search_result.as_ref().unwrap().clone();
                stream(&(search_result[k - 1]))
            }
            Err(e) => println!("Have to be a valid number! {:?}", e),
        }
    }

    parser.save_watched();
}

async fn download_m3u8(files_to_download: Rc<Vec<&M3u8>>) {
    for m3u8 in files_to_download.iter() {
        let file_ending_place = m3u8.link.rfind(".").unwrap();
        let potential_file_ending = &m3u8.link[file_ending_place..];
        let file_ending = if potential_file_ending.len() > 6 {
            ".mkv"
        } else {
            potential_file_ending
        };
        let file_name = format!("{}{}", m3u8.name, file_ending);
        println!("Downloading {}", &file_name);
        if let Err(e) = download_with_progress(&m3u8.link, Some(&file_name)).await {
            eprintln!("Failed to download {}, {:?}", &file_name, e);
        }
    }
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
