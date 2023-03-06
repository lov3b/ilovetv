use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ilovetv")]
pub struct Opt {
    #[structopt(short, long, default_value = "ask")]
    /// Possible options: online, offline and ask
    pub mode: Mode,
}

#[derive(Debug)]
pub enum Mode {
    Online,
    Offline,
    Ask,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Ask
    }
}

impl FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "online" => Ok(Self::Online),
            "offline" => Ok(Self::Offline),
            "ask" | "default" => Ok(Self::Ask),
            _ => Err("No such enum"),
        }
    }
}
