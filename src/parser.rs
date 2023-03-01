use std::fs;
use std::ops::Deref;
use std::rc::Rc;

use crate::m3u8::M3u8;
use crate::Configuration;

const MAX_TRIES: usize = 4;

pub struct Parser {
    configuration: Rc<Configuration>,
    m3u8_items: Vec<M3u8>,
}

impl Parser {
    pub async fn new(configuration: Rc<Configuration>) -> Self {
        let m3u8_items = Self::get_parsed_m3u8(&configuration).await.unwrap();

        Self {
            configuration,
            m3u8_items,
        }
    }

    pub fn find(&self, name: &str) -> Vec<&M3u8> {
        let name = name.to_lowercase();
        self.m3u8_items
            .iter()
            .filter(|item| item.name.to_lowercase().contains(&name) || item.tvg_id.contains(&name))
            .collect()
    }

    pub async fn forcefully_update(&mut self) {
        let mut counter = 0;
        let content = loop {
            counter += 1;
            let content = self.download_playlist().await;
            if counter > MAX_TRIES {
                return;
            } else if content.is_ok() {
                break content.unwrap();
            }
            println!("Retrying {}/{}", counter, MAX_TRIES);
        };

        self.m3u8_items = Self::parse_m3u8(content, &self.seen_links);
    }

    pub fn save_watched(&self) {
        let watched_items = self
            .m3u8_items
            .iter()
            .filter(|item| item.watched)
            .map(|item| item.link.clone())
            .collect::<Vec<String>>();

        let resp = fs::write(
            &self.seen_links_path,
            serde_json::to_string(&watched_items).unwrap(),
        );
        
        if let Err(e) = resp {
            eprintln!("Failed to write watched links {:?}", e);
        }
    }

    fn parse_m3u8(content: String, watched_links: &Vec<String>) -> Vec<M3u8> {
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
            let is_watched = watched_links.contains(link);
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

    async fn get_parsed_m3u8(config: &Configuration) -> Result<Vec<M3u8>, String> {
        Ok(Self::parse_m3u8(
            config.get_playlist().await?,
            &config.seen_links,
        ))
    }
}

impl Deref for Parser {
    type Target = Configuration;

    fn deref(&self) -> &Self::Target {
        &self.configuration
    }
}
