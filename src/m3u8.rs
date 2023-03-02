use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref};

#[derive(Serialize, Deserialize)]
pub struct M3u8 {
    pub tvg_id: String,
    pub tvg_name: String,
    pub tvg_logo: String,
    pub group_title: String,
    pub name: String,
    pub link: String,
    pub watched: bool,
}

impl Display for M3u8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let colored_name = if self.watched {
            self.name.bold().green()
        } else {
            self.name.bold()
        };
        f.write_fmt(format_args!("{} ({})", colored_name, self.link))?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DataEntry {
    m3u8: M3u8,
    pub path: String,
}
impl DataEntry {
    pub fn new(m3u8: M3u8, path: String) -> Self {
        Self { m3u8, path }
    }
}

impl Deref for DataEntry {
    type Target = M3u8;

    fn deref(&self) -> &Self::Target {
        &self.m3u8
    }
}
