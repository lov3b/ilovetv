use crate::M3u8;

pub trait GetM3u8 {
    fn get_m3u8(&self) -> Vec<&M3u8>;
}

pub trait WatchedFind {
    fn find(&self, name: &str) -> Vec<&M3u8>;
    fn get_watched(&self) -> Vec<&M3u8>;
}

impl<T> WatchedFind for T
where
    T: GetM3u8,
{
    fn find(&self, name: &str) -> Vec<&M3u8> {
        let name = name.to_lowercase();
        self.get_m3u8()
            .into_iter()
            .filter(|item| item.name.to_lowercase().contains(&name) || item.tvg_id.contains(&name))
            .collect()
    }

    fn get_watched(&self) -> Vec<&M3u8> {
        self.get_m3u8().into_iter().filter(|x| x.watched).collect()
    }
}
