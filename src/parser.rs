use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use std::{fs, process};

use directories::ProjectDirs;

use crate::m3u8::M3u8;

const MAX_TRIES: usize = 4;

pub struct Parser {
    watched_name: Rc<PathBuf>,
    m3u8_items: Vec<M3u8>,
}

impl Parser {
    fn should_update(file_name: &PathBuf) -> bool {
        fs::metadata(&file_name)
            .and_then(|metadata| {
                Ok({
                    let seconds = metadata.modified()?;
                    seconds
                        .elapsed()
                        .expect("Failed to get systemtime")
                        .as_secs()
                        > 60 * 60 * 24 * 3
                })
            })
            .map_or_else(
                |_| {
                    println!("Could not find playlist-file, Downloading a new one");
                    false
                },
                |x| x,
            )
    }

    pub fn new(file_name: String, iptv_url: String, watched_name: String) -> Self {
        let project_dirs = ProjectDirs::from("com", "billenius", "iptvnator_rs").unwrap();
        let cache = project_dirs.cache_dir();
        let _ = fs::create_dir_all(&cache);

        let file_name = Rc::new(cache.join(file_name));
        let iptv_url = Rc::new(iptv_url);
        let watched_name = Rc::new(cache.join(watched_name));

        Self {
            watched_name: watched_name.clone(),
            m3u8_items: Self::get_parsed_content(&iptv_url, &file_name, &watched_name),
        }
    }

    pub fn find(&self, name: &str) -> Vec<&M3u8> {
        let name = name.to_uppercase();
        self.m3u8_items
            .iter()
            .filter(|item| item.name.to_uppercase().contains(&name) || item.tvg_id.contains(&name))
            .collect()
    }

    pub fn save_watched(&self) {
        let project_dirs = ProjectDirs::from("com", "billenius", "iptvnator_rs").unwrap();
        let cache_dir = project_dirs.cache_dir();
        let watched_cache_file = cache_dir.join(&*self.watched_name);

        let watched_items = self
            .m3u8_items
            .iter()
            .filter(|item| item.watched)
            .map(|item| item.link.clone())
            .collect::<Vec<String>>();

        let _ = fs::create_dir_all(cache_dir);

        match fs::write(watched_cache_file, watched_items.join("\n")) {
            Ok(_) => {
                println!("Saved watched")
            }
            Err(e) => {
                eprintln!("Failed to write downloaded m3u8file {:?}", e);
            }
        }
    }

    fn get_parsed_content(link: &String, file_name: &PathBuf, watched_name: &PathBuf) -> Vec<M3u8> {
        let saved_watches = fs::read_to_string(&watched_name);
        let saved_watches = if saved_watches.is_ok() {
            saved_watches.unwrap()
        } else {
            String::from("")
        };

        let watched: Vec<String> = saved_watches.lines().map(String::from).collect();

        let mut m3u8_items: Vec<M3u8> = Vec::new();
        let interesting_lines: Vec<String> = Self::get_stringcontent(link, file_name, 0)
            .replacen("#EXTM3U\n", "", 1)
            .lines()
            .map(str::trim)
            .map(String::from)
            .collect();

        for i in (0..interesting_lines.len()).step_by(2) {
            let mut items = Vec::new();
            for to_find in ["tvg-id", "tvg-name", "tvg-logo", "group-title"] {
                let offset: usize = format!("{}=", to_find).bytes().len();
                let start: usize =
                    interesting_lines[i].find(&format!("{}=", to_find)).unwrap() as usize + offset;

                let end: usize = interesting_lines[i].rfind("=").unwrap();
                items.push(&interesting_lines[i][start..=end])
            }
            let name_start = interesting_lines[i].rfind(",").unwrap() + 1;
            let name = &interesting_lines[i][name_start..];
            let link = &interesting_lines[i + 1];
            let is_watched = watched.contains(link);
            let m3u8_item = M3u8 {
                tvg_id: items[0].to_owned(),
                tvg_name: items[1].to_owned(),
                tvg_logo: items[2].to_owned(),
                group_title: items[3].to_owned(),
                name: name.to_owned(),
                link: link.to_string(),
                watched: is_watched,
            };
            m3u8_items.push(m3u8_item);
        }
        m3u8_items
    }

    fn get_stringcontent(link: &String, file_name: &PathBuf, tried: usize) -> String {
        if !Self::should_update(file_name) {
            let content = fs::read_to_string(&file_name);
            if content.is_ok() {
                return content.unwrap();
            }
        }

        let content = Self::download(link);
        if content.is_err() && tried < 4 {
            println!("Retrying {}/{}", tried + 1, MAX_TRIES);
            Self::get_stringcontent(link, file_name, tried + 1);
        }

        match content {
            Ok(s) => {
                let _ = fs::write(&file_name, s.as_bytes());
                s
            }
            Err(_) => {
                println!("Couldn't get m3u8 file!");
                process::exit(-1);
            }
        }
    }

    fn download(link: &String) -> Result<String, reqwest::Error> {
        reqwest::blocking::get(link.clone())
            .and_then(|resp| Ok(resp.text().expect("Could not get m3u8 from server")))
    }
}

impl Deref for Parser {
    type Target = Vec<M3u8>;

    fn deref(&self) -> &Self::Target {
        &self.m3u8_items
    }
}
