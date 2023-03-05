use std::{
    borrow::BorrowMut,
    fs,
    ops::{Deref, DerefMut},
    path::PathBuf,
    rc::Rc,
};

use crate::{download_with_progress, downloader::DualWriter, Configuration, Parser, MAX_TRIES};

pub struct Playlist {
    pub content: String,
    path_to_playlist: Rc<PathBuf>,
    url: Rc<String>,
}

impl Playlist {
    pub async fn new(path_to_playlist: Rc<PathBuf>, url: Rc<String>) -> Result<Self, String> {
        let mut me = Self {
            content: String::new(),
            path_to_playlist,
            url,
        };
        me.content = me.get_saved_or_download().await?;

        Ok(me)
    }

    fn get_saved(&self) -> Option<String> {
        if !self.should_update() {
            return fs::read_to_string(&*self.path_to_playlist).ok();
        }
        None
    }

    fn should_update(&self) -> bool {
        fs::metadata(&*self.path_to_playlist)
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

    pub async fn get_saved_or_download(&self) -> Result<String, String> {
        let content = if let Some(content) = self.get_saved() {
            content
        } else {
            let downloaded = self.download().await?;
            if let Err(e) = fs::write(&*self.path_to_playlist, &downloaded) {
                println!(
                    "Failed to save downloaded playlist to file, {:?}, path: '{}'",
                    e,
                    &self.path_to_playlist.as_os_str().to_str().unwrap()
                );
            }
            downloaded
        };

        Ok(content)
    }

    pub async fn download(&self) -> Result<String, String> {
        let mut counter: u8 = 0;
        loop {
            counter += 1;

            let downloaded = download_with_progress(&self.url, None)
                .await
                .and_then(DualWriter::get_string);
            if let Ok(content) = downloaded {
                break Ok(content);
            } else if counter > MAX_TRIES {
                break Err("Failed to download playlist".to_owned());
            }
            println!("Retrying {}/{}", counter + 1, MAX_TRIES);
        }
    }
}

impl Deref for Playlist {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl DerefMut for Playlist {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

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
