use std::{
    fs::{self, File},
    io::{self, BufReader},
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    get_mut_ref, m3u8::OfflineEntry, Readline, APP_IDENTIFIER, JSON_CONFIG_FILENAME,
    STANDARD_OFFLINE_FILENAME, STANDARD_PLAYLIST_FILENAME, STANDARD_SEEN_LINKS_FILENAME,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Conf {
    pub playlist_filename: String,
    pub playlist_url: Rc<String>,
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
        let playlist_url = Self::user_setup().into();

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
    pub playlist_path: Rc<PathBuf>,
    pub seen_links_path: PathBuf,
    pub seen_links: Vec<String>,
    config_file_path: PathBuf,
    pub data_dir: PathBuf,
    pub offlinefile_content: Rc<Vec<OfflineEntry>>,
}

impl Configuration {
    pub fn new() -> Result<Self, io::Error> {
        let project_dirs =
            ProjectDirs::from(APP_IDENTIFIER[0], APP_IDENTIFIER[1], APP_IDENTIFIER[2]).unwrap();

        // Make sure all the dirs for the project are setup correctly
        let config_dir = project_dirs.config_dir();
        let cache_dir = project_dirs.cache_dir().to_path_buf();
        let offline_dir = project_dirs.data_local_dir().to_path_buf();
        for dir in [&config_dir, &cache_dir.as_path(), &offline_dir.as_path()].iter() {
            if !dir.exists() {
                let _ = fs::create_dir_all(dir);
            }
        }

        // Config setup
        let config_file_path = config_dir.join(JSON_CONFIG_FILENAME).to_path_buf();
        let configuration = Conf::new(&config_file_path)?;
        fs::write(
            &config_file_path,
            serde_json::to_string(&configuration).unwrap(),
        )?;

        // Playlist
        let playlist_path = cache_dir.join(&configuration.playlist_filename).into();
        let seen_links_path = cache_dir.join(&configuration.seen_links_filename);
        let seen_links = Self::get_watched(&seen_links_path).unwrap_or_default();

        // Datadir
        let offlinefile = offline_dir.join(STANDARD_OFFLINE_FILENAME);
        let offlinefile_content =
            Rc::new(Self::get_offline_content(&offlinefile).unwrap_or_default());

        Ok(Self {
            conf: configuration,
            playlist_path,
            seen_links,
            seen_links_path,
            config_file_path,
            data_dir: offline_dir,
            offlinefile_content,
        })
    }

    pub fn update_last_search_ugly(&self, last_search: Option<String>) {
        unsafe { get_mut_ref(&self.conf).last_search = last_search }

        if let Err(e) = self.write_configfile(&self.config_file_path) {
            println!("Failed to write to configfile, {:?}", e);
        }
    }

    pub fn push_offlinefile_ugly(&self, data_entry: OfflineEntry) {
        unsafe { get_mut_ref(&*self.offlinefile_content) }.push(data_entry);
    }

    pub fn write_datafile(&self) -> Result<(), io::Error> {
        let path = self.data_dir.join(STANDARD_OFFLINE_FILENAME);
        fs::write(path, serde_json::to_string(&self.offlinefile_content)?)
    }

    fn get_watched(path: &Path) -> Option<Vec<String>> {
        let reader = BufReader::new(File::open(&path).ok()?);
        serde_json::from_reader(reader).ok()
    }

    fn get_offline_content(datafile: &PathBuf) -> Option<Vec<OfflineEntry>> {
        let reader = BufReader::new(File::open(datafile).ok()?);
        serde_json::from_reader(reader).ok()
    }
}

impl Deref for Configuration {
    type Target = Conf;

    fn deref(&self) -> &Self::Target {
        &self.conf
    }
}
