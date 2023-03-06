#[allow(unused_imports)]
use crate::GetM3u8;
use crate::{
    get_mut_ref,
    parser::{Parser, WatchedFind},
    Configuration, OfflineParser, OnlineParser, Playlist, MAX_TRIES,
};
use std::{fs, rc::Rc};

type Error = String;

pub struct GrandMother {
    pub parser: Box<dyn Parser>,
    pub playlist: Option<Playlist>,
    pub config: Rc<Configuration>,
}

impl GrandMother {
    pub async fn new_online(config: Rc<Configuration>) -> Result<Self, Error> {
        let playlist = Playlist::new(config.playlist_path.clone(), config.playlist_url.clone());
        let seen_links = config.seen_links.iter().map(|x| x.as_str()).collect();
        let playlist = playlist.await?;
        let playlist_content = playlist.get_saved_or_download().await?;
        let parser: Box<dyn Parser> =
            Box::new(OnlineParser::new(&playlist_content, &seen_links).await);

        Ok(Self {
            parser,
            playlist: Some(playlist),
            config,
        })
    }

    pub async fn promote_to_online(&mut self) -> Result<(), Error> {
        let online_mother = GrandMother::new_online(self.config.clone()).await?;
        (self.parser, self.playlist) = (online_mother.parser, online_mother.playlist);

        Ok(())
    }

    pub fn new_offline(config: Rc<Configuration>) -> Self {
        let parser: Box<dyn Parser> = Box::new(OfflineParser::new(&config));
        Self {
            parser,
            playlist: None,
            config,
        }
    }

    pub async fn refresh_dirty(&self) -> Result<(), Error> {
        unsafe { get_mut_ref(self) }.refresh().await
    }

    pub async fn refresh(&mut self) -> Result<(), Error> {
        let mut counter = 0;
        let content = loop {
            counter += 1;
            let content = self
                .playlist
                .as_ref()
                .ok_or_else(|| "Cannot refresh playlist in offlinemode")?
                .download()
                .await;
            if counter > MAX_TRIES {
                return Ok(());
            }
            if let Ok(content) = content {
                break content;
            }
            println!("Retrying {}/{}", counter, MAX_TRIES);
        };

        let watched_links = self
            .parser
            .get_watched()
            .iter()
            .map(|x| x.link.as_str())
            .collect();
        self.parser = Box::new(OnlineParser::new(&content, &watched_links).await);

        Ok(())
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
