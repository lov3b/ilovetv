use std::fs;

use crate::{Configuration, Parser, Playlist, MAX_TRIES};

pub struct GrandMother {
    pub parser: Parser,
    pub playlist: Playlist,
    pub config: Configuration,
}

impl GrandMother {
    pub async fn new(config: Configuration) -> Result<Self, String> {
        let playlist = Playlist::new(config.playlist_path.clone(), config.playlist_url.clone());
        let seen_links = config.seen_links.iter().map(|x| x.as_str()).collect();
        let playlist = playlist.await?;
        let playlist_content = playlist.get_saved_or_download().await?;

        let parser = Parser::new(&playlist_content, &seen_links).await;

        Ok(Self {
            parser,
            playlist,
            config,
        })
    }

    pub async fn refresh_dirty(&self) {
        let ptr = self as *const Self as *mut Self;
        unsafe { &mut *ptr }.refresh().await;
    }

    pub async fn refresh(&mut self) {
        let mut counter = 0;
        let content = loop {
            counter += 1;
            let content = self.playlist.download().await;
            if counter > MAX_TRIES {
                return;
            }
            if let Ok(content) = content {
                break content;
            }
            println!("Retrying {}/{}", counter, MAX_TRIES);
        };

        let watched_links = self.parser.get_watched();
        let watched_links = watched_links.iter().map(|x| x.as_str()).collect();
        self.parser = Parser::new(&content, &watched_links).await;
    }

    pub fn save_watched(&self) {
        let watched_items = self.parser.get_watched();

        let resp = fs::write(
            &self.config.seen_links_path,
            serde_json::to_string(&watched_items).unwrap(),
        );

        if let Err(e) = resp {
            eprintln!("Failed to write watched links {:?}", e);
        }
    }
}
