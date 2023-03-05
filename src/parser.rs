use std::{
    fs::{self, File},
    io::BufReader,
    ops::Deref,
};

use serde::Serialize;

use crate::{m3u8::M3u8, Configuration, GetM3u8};

pub struct Parser {
    m3u8_items: Vec<M3u8>,
}

impl Parser {
    pub async fn new(m3u_content: &str, watched_links: &Vec<&str>) -> Self {
        Self {
            m3u8_items: Self::parse_m3u8(m3u_content, watched_links),
        }
    }

    pub fn find(&self, name: &str) -> Vec<&M3u8> {
        let name = name.to_lowercase();
        self.m3u8_items
            .iter()
            .filter(|item| item.name.to_lowercase().contains(&name) || item.tvg_id.contains(&name))
            .collect()
    }

    /*
     * I know that this is also frowned upon, but it is perfectly safe right here,
     * even though the borrowchecker complains
     */
    // async fn refresh(&self) {
    //     unsafe { get_mut_ref(&self.pla) }.forcefully_update().await;
    // }

    pub async fn forcefully_update(&mut self, content: &str) {
        let seen_links: &Vec<&str> = &self
            .m3u8_items
            .iter()
            .filter(|x| x.watched)
            .map(|x| x.link.as_str())
            .collect();

        self.m3u8_items = Self::parse_m3u8(content, seen_links);
    }

    fn parse_m3u8(content: &str, watched_links: &Vec<&str>) -> Vec<M3u8> {
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
            let link = interesting_lines[i + 1].as_str();
            let is_watched = watched_links.contains(&link);
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
}

impl Deref for Parser {
    type Target = Vec<M3u8>;

    fn deref(&self) -> &Self::Target {
        &self.m3u8_items
    }
}

impl GetM3u8 for Parser {
    fn get_m3u8(&self) -> Vec<&M3u8> {
        self.m3u8_items.iter().collect()
    }
}

#[derive(Serialize)]
struct OfflineEntry {
    m3u8: M3u8,
    path: String,
}
#[derive(Serialize)]
struct OfflineParser {
    m3u8_items: Vec<OfflineEntry>,
}
impl OfflineParser {
    pub fn new(config: &Configuration) -> Result<Self, std::io::Error> {
        todo!()
    }
}
