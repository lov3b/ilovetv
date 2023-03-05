use std::{
    fs,
    ops::{Deref, DerefMut},
    path::PathBuf,
    rc::Rc,
};

use crate::{download_with_progress, downloader::DualWriter, MAX_TRIES};

pub struct Playlist {
    pub content: String,
    path_to_playlist: Rc<PathBuf>,
    url: Option<Rc<String>>,
}

impl Playlist {
    pub async fn new(path_to_playlist: Rc<PathBuf>, url: Rc<String>) -> Result<Self, String> {
        let mut me = Self {
            content: String::new(),
            path_to_playlist,
            url: Some(url),
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

            let url = self
                .url
                .as_ref()
                .clone()
                .ok_or_else(|| String::from("In offline mode"))?;

            let downloaded = download_with_progress(url, None)
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
