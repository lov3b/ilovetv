use std::{ops::Deref, rc::Rc};

use serde::Serialize;

use crate::{m3u8::M3u8, Configuration, GetM3u8, GetPlayPath, OfflineEntry};

#[derive(Serialize)]
pub struct OfflineParser {
    m3u8_items: Rc<Vec<OfflineEntry>>,
}
impl OfflineParser {
    pub fn new(config: &Configuration) -> Self {
        Self {
            m3u8_items: config.offlinefile_content.clone(),
        }
    }
}

impl Deref for OfflineParser {
    type Target = Vec<OfflineEntry>;

    fn deref(&self) -> &Self::Target {
        &*self.m3u8_items
    }
}

impl GetPlayPath for OfflineParser {
    fn get_path_to_play(&self, link: Rc<String>) -> Result<Rc<String>, String> {
        for offline_entry in &*self.m3u8_items {
            if *offline_entry.link == *link {
                return Ok(offline_entry.path.clone());
            }
        }
        Err("Not stored for offline use".to_owned())
    }
}

impl GetM3u8 for OfflineParser {
    fn get_m3u8(&self) -> Vec<&M3u8> {
        self.m3u8_items.iter().map(|x| &**x).collect()
    }
}
