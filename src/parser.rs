use std::path::PathBuf;
use std::rc::Rc;
use std::{fs, process};

use directories::ProjectDirs;

use crate::downloader::download_with_progress;
use crate::m3u8::M3u8;

const MAX_TRIES: usize = 4;

pub struct Parser {
    watched_name: Rc<PathBuf>,
    m3u8_items: Vec<M3u8>,
    ilovetv_url: Rc<String>,
    file_name: Rc<PathBuf>,
}

impl Parser {
    pub async fn new(file_name: String, iptv_url: String, watched_name: String) -> Self {
        let project_dirs = ProjectDirs::from("com", "billenius", "iptvnator_rs").unwrap();
        let cache = project_dirs.cache_dir();
        let _ = fs::create_dir_all(&cache);

        let file_name = Rc::new(cache.join(file_name));
        let ilovetv_url = Rc::new(iptv_url);
        let watched_name = Rc::new(cache.join(watched_name));

        Self {
            watched_name: watched_name.clone(),
            m3u8_items: Self::get_parsed_content(&ilovetv_url, &file_name, &watched_name).await,
            ilovetv_url,
            file_name,
        }
    }

    pub fn find(&self, name: &str) -> Vec<&M3u8> {
        let name = name.to_lowercase();
        self.m3u8_items
            .iter()
            .filter(|item| item.name.to_lowercase().contains(&name) || item.tvg_id.contains(&name))
            .collect()
    }

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

    pub async fn forcefully_update(&mut self) {
        let mut counter = 0;
        let content = loop {
            counter += 1;
            let content = Self::download(&self.ilovetv_url).await.ok();
            if counter > MAX_TRIES {
                return;
            } else if content.is_some() {
                break content.unwrap();
            }
            println!("Retrying {}/{}", counter, MAX_TRIES);
        };

        let _ = fs::write(&*self.file_name, &content);
        self.m3u8_items = Self::parse_m3u8(content, &self.watched_name.clone());
    }

    pub fn save_watched(&self) {
        let watched_items = self
            .m3u8_items
            .iter()
            .filter(|item| item.watched)
            .map(|item| item.link.clone())
            .collect::<Vec<String>>();

        let _ = fs::create_dir_all(&*self.watched_name.parent().unwrap());

        match fs::write(&*self.watched_name, watched_items.join("\n")) {
            Ok(_) => {
                println!("Saved watched")
            }
            Err(e) => {
                eprintln!("Failed to write downloaded m3u8file {:?}", e);
            }
        }
    }

    async fn get_parsed_content(
        link: &String,
        file_name: &PathBuf,
        watched_name: &PathBuf,
    ) -> Vec<M3u8> {
        Self::parse_m3u8(
            Self::get_stringcontent(link, file_name)
                .await
                .expect("Failed to retrieve playlist"),
            watched_name,
        )
    }

    fn parse_m3u8(content: String, watched_name: &PathBuf) -> Vec<M3u8> {
        let saved_watches = fs::read_to_string(&watched_name);
        let saved_watches = if saved_watches.is_ok() {
            saved_watches.unwrap()
        } else {
            String::from("")
        };

        let watched: Vec<String> = saved_watches.lines().map(String::from).collect();

        let mut m3u8_items: Vec<M3u8> = Vec::new();
        let interesting_lines: Vec<String> = content
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

    async fn get_stringcontent(link: &String, file_name: &PathBuf) -> Result<String, String> {
        if !Self::should_update(file_name) {
            let content = fs::read_to_string(&file_name);
            if content.is_ok() {
                return Ok(content.unwrap());
            }
        }

        let mut counter: usize = 0;
        let content = loop {
            counter += 1;

            if let Ok(content) = Self::download(link).await {
                break Ok(content);
            } else if counter > MAX_TRIES {
                break Err("".to_owned());
            }
            println!("Retrying {}/{}", counter + 1, MAX_TRIES);
        };

        match content {
            Ok(s) => {
                let _ = fs::write(&file_name, s.as_bytes());
                Ok(s)
            }
            Err(_) => {
                println!("Couldn't get m3u8 file!");
                process::exit(-1);
            }
        }
    }

    async fn download(link: &String) -> Result<String, String> {
        Ok(download_with_progress(link, None)
            .await?
            .get_string()
            .unwrap())
    }
}
