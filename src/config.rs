use std::{
    fs::{self, File},
    io::{self, BufReader},
    ops::Deref,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{download_with_progress, get_mut_ref, Readline};

const JSON_CONFIG_FILENAME: &'static str = "iptvnator_config.json";
const APP_IDENTIFIER: [&'static str; 3] = ["com", "billenius", "iptvnator"];
const STANDARD_PLAYLIST_FILENAME: &'static str = "ilovetv.m3u8";
const STANDARD_SEEN_LINKS_FILENAME: &'static str = "watched_links.json";
const MAX_TRIES: u8 = 4;

#[derive(Serialize, Deserialize, Debug)]
pub struct Conf {
    pub playlist_filename: String,
    pub playlist_url: String,
    pub last_search: Option<String>,
    pub seen_links_filename: String,
}

impl Conf {
    /**
     * Read configurationfile or ask user for link input if it isn't created.
     * Will error if it fails to write config file
     */
    pub fn new(ilovetv_config_file: &Path) -> Result<Conf, io::Error> {
        // Read the configuraionfile if it exists
        if ilovetv_config_file.exists() {
            let config_file = Self::read_configfile(&ilovetv_config_file);
            if let Ok(cfg) = config_file {
                return Ok(cfg);
            } else {
                println!("There are some problem with the configurationfile");
            }
        }

        // Get fresh config with url from user
        let playlist_url = Self::user_setup();

        Ok(Self {
            playlist_filename: STANDARD_PLAYLIST_FILENAME.to_owned(),
            playlist_url,
            last_search: None,
            seen_links_filename: STANDARD_SEEN_LINKS_FILENAME.to_owned(),
        })
    }

    fn read_configfile(config_file: &Path) -> Result<Conf, io::Error> {
        let reader = BufReader::new(File::open(config_file)?);
        let conf: Conf = serde_json::from_reader(reader)?;
        Ok(conf)
    }

    fn write_configfile(&self, path: &Path) -> Result<(), io::Error> {
        fs::write(path, serde_json::to_string(&self)?)?;
        Ok(())
    }

    fn user_setup() -> String {
        let mut readline = Readline::new();

        println!("Hello, I would need an url to your iptv/m3u/m3u8 stream");
        loop {
            let url = readline.input("enter url: ");
            let yn = readline.input("Are you sure? (Y/n) ");

            if yn.trim().to_lowercase() != "n" {
                break url.trim().to_owned();
            }
        }
    }
}

pub struct Configuration {
    pub conf: Conf,
    pub playlist_path: PathBuf,
    pub seen_links_path: PathBuf,
    pub seen_links: Vec<String>,
    config_file_path: PathBuf,
}

impl Configuration {
    pub fn new() -> Result<Self, io::Error> {
        let project_dirs =
            ProjectDirs::from(APP_IDENTIFIER[0], APP_IDENTIFIER[1], APP_IDENTIFIER[2]).unwrap();
        let config_dir = project_dirs.config_dir();
        let _ = fs::create_dir_all(config_dir);
        let config_file_path = config_dir.join(JSON_CONFIG_FILENAME).to_path_buf();

        let configuration = Conf::new(&config_file_path)?;

        fs::write(
            &config_file_path,
            serde_json::to_string(&configuration).unwrap(),
        )?;

        // Setup dirs for playlist
        let cache_dir = project_dirs.cache_dir().to_path_buf();
        let playlist_path = cache_dir.join(&configuration.playlist_filename);
        let seen_links_path = cache_dir.join(&configuration.seen_links_filename);
        let _ = fs::create_dir_all(&cache_dir);

        let seen_links = Self::get_watched(&seen_links_path).unwrap_or_default();

        Ok(Self {
            conf: configuration,
            playlist_path,
            seen_links,
            seen_links_path,
            config_file_path,
        })
    }

    pub fn update_last_search_ugly(&self, last_search: Option<String>) {
        unsafe { get_mut_ref(&self.conf).last_search = last_search }

        if let Err(e) = self.write_configfile(&self.config_file_path) {
            println!("Failed to write to configfile, {:?}", e);
        }
    }

    fn get_watched(path: &Path) -> Option<Vec<String>> {
        let reader = BufReader::new(File::open(&path).ok()?);
        serde_json::from_reader(reader).ok()
    }

    fn should_update_playlist(&self) -> bool {
        fs::metadata(&self.playlist_path)
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
    pub async fn get_playlist(&self) -> Result<String, String> {
        let content = if let Some(content) = self.get_saved_playlist() {
            content
        } else {
            let downloaded = self.download_playlist().await?;
            if let Err(e) = fs::write(&self.playlist_path, &downloaded) {
                println!(
                    "Failed to save downloaded playlist to file, {:?}, path: '{}'",
                    e,
                    &self.playlist_path.as_os_str().to_str().unwrap()
                );
            }
            downloaded
        };

        Ok(content)
    }

    fn get_saved_playlist(&self) -> Option<String> {
        if !self.should_update_playlist() {
            return fs::read_to_string(&self.playlist_path).ok();
        }
        None
    }

    pub async fn download_playlist(&self) -> Result<String, String> {
        let mut counter: u8 = 0;
        loop {
            counter += 1;

            if let Ok(content) = self.just_download().await {
                break Ok(content);
            } else if counter > MAX_TRIES {
                break Err("Failed to download playlist".to_owned());
            }
            println!("Retrying {}/{}", counter + 1, MAX_TRIES);
        }
    }

    async fn just_download(&self) -> Result<String, String> {
        download_with_progress(&self.playlist_url, None)
            .await?
            .get_string()
    }
}

impl Deref for Configuration {
    type Target = Conf;

    fn deref(&self) -> &Self::Target {
        &self.conf
    }
}
