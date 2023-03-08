use std::num::ParseIntError;
use std::process::Command;
use std::rc::Rc;

use colored::Colorize;
use structopt::StructOpt;

use ilovetv::{
    download_with_progress, get_gm, get_mut_ref, Configuration, M3u8, OfflineEntry, Opt, Readline,
    WatchedFind,
};
#[allow(unused_imports)]
use ilovetv::{GetM3u8, GetPlayPath, OfflineParser};

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    // Greet the user
    [
        format!(
            "Welcome to {}, a {} iptv client written in rust\n",
            "ilovetv".bold(),
            "BLAZINGLY FAST".italic()
        ),
        "There will be some options along the way".to_owned(),
        format!(" {} is to refresh the local iptvfile.", "r".bold()),
        format!(" {} is to quit and save watched fields", "q".bold()),
        format!(" {} is to download fields", "d".bold()),
        format!(
            " {} is to make entries availibe for offline use later on in the program",
            "o".bold()
        ),
        format!(" {} is to switch between modes (toggle)", "m".bold()),
        format!(" {} is to perform a new search", "s".bold()),
        format!(" {} is to select all", "a".bold()),
        format!(" {} is to toggtle fullscreen for mpv", "f".bold()),
        format!(
            " {} is to redo the last search (mainly for use in the last session)",
            "l".bold()
        ),
        format!(" {} is to clean the latest search", "c".bold()),
    ]
    .iter()
    .for_each(|s| println!("{}", &s));

    let mut mpv_fs = false;
    let mut search_result: Option<Rc<Vec<&M3u8>>> = None;
    let mut readline = Readline::new();
    let (gm, mut in_online) = get_gm(
        opt.mode,
        &mut readline,
        Rc::new(Configuration::new().expect("Failed to write to configfile")),
    )
    .await
    .expect("Failed to retrive online playlist");

    loop {
        // Dont't perform a search if user has just watched, instead present the previous search
        if search_result.is_none() {
            let search = readline
                .input("Search by name [ r/q/f/l/m ]: ")
                .to_lowercase();
            let mut search = search.trim();

            // Special commands
            match search {
                // Quit
                "q" => break,
                // Refresh playlist
                "r" => {
                    search_result = None;
                    if let Err(e) = gm.refresh_dirty().await {
                        println!(
                            "Cannot refresh. This is probably due to offlinemode {:?}",
                            e
                        );
                    };
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
                "l" => {
                    search = if let Some(s) = gm.config.last_search.as_ref() {
                        s
                    } else {
                        println!("There is no search saved from earlier");
                        continue;
                    };
                }
                "c" => {
                    gm.config.update_last_search_ugly(None);
                    continue;
                }
                "m" => {
                    if in_online {
                        unsafe { get_mut_ref(&gm) }.demote_to_offline();
                        println!("Switched to offline mode");
                    } else {
                        let result = unsafe { get_mut_ref(&gm) }.promote_to_online().await;
                        if let Err(e) = result {
                            println!("Failed to switch to onlinemode {:?}", e);
                        } else {
                            println!("Switched to online mode");
                            continue;
                        }
                    }
                    in_online = !in_online;
                    continue;
                }
                _ => {}
            }
            search_result = Some(Rc::new(gm.parser.find(search)));

            if search_result.as_ref().unwrap().is_empty() {
                println!("Nothing found");
                search_result = None;
                continue;
            }
            gm.config.update_last_search_ugly(Some(search.to_owned()));
        }

        // Let them choose which one to stream
        for (idx, m3u8_item) in search_result.as_ref().unwrap().iter().enumerate().rev() {
            println!("  {}: {}", idx + 1, m3u8_item);
        }

        let user_wish = readline
            .input("Which one do you wish to stream? [ q/f/s/r/d/o/m ]: ")
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
                if let Err(e) = gm.refresh_dirty().await {
                    println!(
                        "Cannot refresh. This is probably due to offlinemode {:?}",
                        e
                    );
                };
                continue;
            }
            "f" => {
                mpv_fs = !mpv_fs;
                println!(
                    "Toggled mpv to {}launch in fullscreen",
                    if mpv_fs { "" } else { "not " }
                );
                continue;
            }
            // Downloadmode
            "d" => {
                let download_selections =
                    ask_which_to_download(&mut readline, &search_result.as_ref().unwrap());

                for to_download in download_selections.iter() {
                    download_m3u8(to_download, None).await;
                }
                continue;
            }
            // Save to offlinemode
            "o" => {
                let download_selections =
                    ask_which_to_download(&mut readline, &search_result.as_ref().unwrap());

                for to_download in download_selections.iter() {
                    let file_ending = to_download
                        .name
                        .rfind(".")
                        .map(|dot_idx| {
                            if dot_idx < 6 {
                                &to_download.name[dot_idx..]
                            } else {
                                ".mkv"
                            }
                        })
                        .unwrap_or_else(|| ".mkv");

                    let path = gm
                        .config
                        .data_dir
                        .join(format!("{}{}", &to_download.name, file_ending));
                    let path = Rc::new(path.to_string_lossy().to_string());
                    download_m3u8(to_download, Some(&path)).await;
                    let data_entry = OfflineEntry::new((*to_download).clone(), path);
                    gm.config.push_offlinefile_ugly(data_entry);
                }

                if let Err(e) = gm.config.write_datafile() {
                    println!(
                        "Failed to information about downloaded entries for offline use {:?}",
                        e
                    )
                }
                continue;
            }
            "m" => {
                let result = unsafe { get_mut_ref(&gm) }.promote_to_online().await;
                if let Err(e) = result {
                    println!("Failed to switch to onlinemode {:?}", e);
                }
                continue;
            }
            _ => {}
        }

        let choosen = user_wish.parse::<usize>();
        match choosen {
            Ok(k) => {
                let search_result = search_result.as_ref().unwrap();
                let to_play = search_result[k - 1];
                let path_link = if let Ok(link) = gm.parser.get_path_to_play(to_play.link.clone()) {
                    link
                } else {
                    println!("Not possible to refresh playlist while in offlinemode");
                    continue;
                };
                stream(to_play, &*path_link, mpv_fs);
                gm.save_watched();
            }
            Err(e) => println!("Have to be a valid number! {:?}", e),
        }
    }
}

fn ask_which_to_download<'a>(
    readline: &mut Readline,
    search_result: &Rc<Vec<&'a M3u8>>,
) -> Rc<Vec<&'a M3u8>> {
    let selections = loop {
        // Ask for userinput
        let selection = readline
            .input("Download all or select in comma separated [a | 1,2,3,4]: ")
            .to_lowercase();
        let selection = selection.trim();

        // Download all
        if selection == "a" {
            println!("Downloading all");
            return search_result.clone();
        }

        // Convert to numbers
        let selections = selection
            .split(",")
            .map(|x| x.trim().parse::<usize>())
            .collect::<Vec<Result<usize, ParseIntError>>>();

        // Ask again if any number wasn't a valid number
        let wrong_input = selections.iter().any(|x| x.is_err());
        if wrong_input {
            println!("Invalid input. Has to be either {}, a number or a sequence of numbers separated by commas","a".bold());
            continue;
        }

        break selections;
    };

    Rc::new(
        selections
            .into_iter()
            .map(|x| x.unwrap() - 1) // Since all numbers are valid, remap them
            .map(|x| search_result[x]) // We don't want the numbers, but the &M3u8 in those positions
            .collect(),
    )
}

async fn download_m3u8(file_to_download: &M3u8, path: Option<&str>) {
    let file_ending_place = file_to_download.link.rfind(".").unwrap();
    let potential_file_ending = &file_to_download.link[file_ending_place..];
    let file_ending = if potential_file_ending.len() > 6 {
        ".mkv"
    } else {
        potential_file_ending
    };
    let file_name = format!("{}{}", file_to_download.name, file_ending);
    println!("Downloading {}", &file_name);
    let path = if let Some(path) = path {
        format!("{}{}", path, file_ending)
    } else {
        file_name.clone()
    };

    if let Err(e) = download_with_progress(&file_to_download.link, Some(&path)).await {
        eprintln!("Failed to download {}, {:?}", &file_name, e);
    }
}

/**
 * I know that this is not how youre supposed to do things, but it's perfectly safe
 * in this context and also the most efficient way.
 * With other words, it's BLAZINGLY FAST
 */
fn stream(m3u8item: &M3u8, link: &String, launch_in_fullscreen: bool) {
    let mut m3u8item = unsafe { get_mut_ref(m3u8item) };
    m3u8item.watched = true;
    let mut args: Vec<&str> = vec![link];
    if launch_in_fullscreen {
        args.push("--fs");
    }

    Command::new("mpv")
        .args(args)
        .output()
        .expect("Could not listen for output");
}
