use colored::Colorize;
use std::fmt::Display;

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
        f.write_fmt(format_args!("{}({})", colored_name, self.link))?;
        Ok(())
    }
}
