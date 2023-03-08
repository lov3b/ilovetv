use std::rc::Rc;

use crate::M3u8;

pub trait GetM3u8 {
    fn get_m3u8(&self) -> Vec<&M3u8>;
}

pub trait WatchedFind {
    fn find(&self, name: &str) -> Vec<&M3u8>;
    fn get_watched_links(&self) -> Vec<Rc<String>>;
}

impl<T: ?Sized + GetM3u8> WatchedFind for T {
    fn find(&self, name: &str) -> Vec<&M3u8> {
        let name = name.to_lowercase();
        self.get_m3u8()
            .into_iter()
            .filter(|item| item.name.to_lowercase().contains(&name) || item.tvg_id.contains(&name))
            .collect()
    }

    fn get_watched_links(&self) -> Vec<Rc<String>> {
        self.get_m3u8()
            .into_iter()
            .filter(|x| x.watched)
            .map(|x| x.link.clone())
            .collect()
    }
}
pub trait GetPlayPath {
    fn get_path_to_play(&self, link: Rc<String>) -> Result<Rc<String>, String>;
}

pub trait Parser: GetM3u8 + GetPlayPath {}
impl<T: GetM3u8 + GetPlayPath> Parser for T {}
