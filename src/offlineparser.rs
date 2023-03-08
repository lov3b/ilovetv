use std::{ops::Deref, rc::Rc};

use serde::Serialize;

use crate::{m3u8::M3u8, Configuration, GetM3u8, GetPlayPath, OfflineEntry};

#[derive(Serialize)]
pub struct OfflineParser {
    offline_entries: Rc<Vec<OfflineEntry>>,
}
impl OfflineParser {
    pub fn new(config: &Configuration) -> Self {
        Self {
            offline_entries: config.offlinefile_content.clone(),
        }
    }
}

impl Deref for OfflineParser {
    type Target = Vec<OfflineEntry>;

    fn deref(&self) -> &Self::Target {
        &*self.offline_entries
    }
}

impl GetPlayPath for OfflineParser {
    fn get_path_to_play(&self, link: Rc<String>) -> Result<Rc<String>, String> {
        for offline_entry in &*self.offline_entries {
            if *offline_entry.link == *link {
                return Ok(offline_entry.path.clone());
            }
        }
        Err("Not stored for offline use".to_owned())
    }
}

impl GetM3u8 for OfflineParser {
    fn get_m3u8(&self) -> Vec<&M3u8> {
        let mut items: Vec<&M3u8> = self.offline_entries.iter().map(|x| &**x).collect();
        items.sort_by_key(|x| &x.link);
        items
    }
}
