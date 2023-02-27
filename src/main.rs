use std::num::ParseIntError;
use std::process::Command;
use std::rc::Rc;

use colored::Colorize;
use iptvnator::{download_with_progress, setup, M3u8, Parser, Readline};

#[tokio::main]
async fn main() {
    println!(
        "Welcome to {}, a {} iptv client written in rust\n",
        "iptvnator".bold(),
        "BLAZINGLY FAST".italic()
    );
    println!(
        "There will be some options along the way \n {} is to refresh the local iptvfile.\n {} is to quit and save watched fields\n {} is to download fields\n {} is to perform a new search\n {} is to select all\n {} is to toggtle fullscreen for mpv",
        "r".bold(),"q".bold(),"d".bold(),"s".bold(),"a".bold(), "f".bold()
    );

    let parser = Parser::new("iptv.m3u8".to_owned(), setup(), "watched.txt".to_owned()).await;

    let mut mpv_fs = false;
    let mut search_result: Option<Rc<Vec<&M3u8>>> = None;
    let mut readline = Readline::new();

    loop {
        // Dont't perform a search if user has just watched, instead present the previous search
        if search_result.is_none() {
            let search = readline.input("Search by name [ r/q/f ]: ").to_lowercase();
            let search = search.trim();

            // Special commands
            match search {
                // Quit
                "q" => break,
                // Refresh playlist
                "r" => {
                    search_result = None;
                    refresh(&parser).await;
                    continue;
                }
                // Toggle fullscreen for mpv
                "f" => {
                    mpv_fs = !mpv_fs;
                    println!(
                        "Toggled mpv to {}launch in fullscreen",
                        if mpv_fs { "" } else { "not " }
                    );
                    continue;
                }
                _ => {}
            }
            search_result = Some(Rc::new(parser.find(search)));

            if search_result.as_ref().unwrap().len() == 0 {
                println!("Nothing found");
                continue;
            }
        }

        // Let them choose which one to stream
        for (idx, m3u8_item) in search_result.as_ref().unwrap().iter().enumerate().rev() {
            println!("  {}: {}", idx + 1, m3u8_item);
        }

        let user_wish = readline
            .input("Which one do you wish to stream? [ q/s/r/d ]: ")
            .to_lowercase();
        let user_wish = user_wish.trim();

        // If they want to quit, let them-
        match user_wish {
            // Quit
            "q" => break,
            // Go inte search-mode
            "s" => {
                search_result = None;
                continue;
            }
            // Refresh playlist
            "r" => {
                println!("Refreshing local m3u8-file");
                search_result = None;
                refresh(&parser).await;
                continue;
            }
            // Downloadmode
            "d" => {
                let selection = readline
                    .input("Download all or select in comma separated [a | 1,2,3,4]: ")
                    .to_lowercase();
                let selection = selection.trim();

                let to_download = loop {
                    break if selection == "a" {
                        println!("Downloading all");
                        search_result.as_ref().unwrap().clone()
                    } else {
                        let selections = selection
                            .split(",")
                            .map(|x| x.trim().parse::<usize>())
                            .collect::<Vec<Result<usize, ParseIntError>>>();

                        for selection in selections.iter() {
                            if selection.is_err() {
                                println!("Not a valid number or the option {}", "a".bold());
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
            _ => {}
        }

        let choosen = user_wish.parse::<usize>();
        match choosen {
            Ok(k) => {
                let search_result = search_result.as_ref().unwrap().clone();
                stream(&(search_result[k - 1]), mpv_fs);
                parser.save_watched();
            }
            Err(e) => println!("Have to be a valid number! {:?}", e),
        }
    }
}

/*
 * I know that this is also frowned upon, but it is perfectly safe right here,
 * even though the borrowchecker complains
 */
async fn refresh(parser: &Parser) {
    let ptr = parser as *const Parser as *mut Parser;
    let p = unsafe { &mut *ptr };
    p.forcefully_update().await;
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

/**
 * This function uses unsafe code to change an atribute, and while I know that this is not
 * how youre supposed to do things, it's perfectly safe in this context and also the most efficient way.
 * With other words, it's BLAZINGLY FAST
 */
fn stream(m3u8item: &M3u8, launch_in_fullscreen: bool) {
    let ptr = m3u8item as *const M3u8 as *mut M3u8;
    let mut item = unsafe { &mut *ptr };
    item.watched = true;
    let mut args: Vec<&str> = vec![&m3u8item.link];
    if launch_in_fullscreen {
        args.push("--fs");
    }

    Command::new("mpv")
        .args(args)
        .output()
        .expect("Could not listen for output");
}
