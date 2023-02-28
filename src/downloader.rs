use std::{
    fmt,
    fs::File,
    io::{self, Read, Write},
};

use bytes::Bytes;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::{self, Client};
use std::cmp;

use crate::get_mut_ref;

pub enum DualWriter {
    File(File),
    Buffer(Vec<u8>),
}

impl DualWriter {
    pub fn write(&mut self, bytes: Bytes) -> Result<(), std::io::Error> {
        match self {
            Self::Buffer(buffer) => {
                bytes.into_iter().for_each(|byte| buffer.push(byte));
            }
            Self::File(file) => {
                file.write(&bytes)?;
            }
        }

        Ok(())
    }

    pub fn get_string(self) -> Result<String, String> {
        Ok(match self {
            Self::Buffer(buffer) => {
                String::from_utf8(buffer).or(Err("Failed to decode buffer".to_owned()))?
            }
            Self::File(file) => {
                let mut buf = String::new();

                // Well this is safe since I consume the file anyways
                let file = unsafe { get_mut_ref(&file) };
                file.read_to_string(&mut buf)
                    .or(Err("Failed to read file".to_owned()))?;
                buf
            }
        })
    }

    pub fn new(file_name: Option<&str>) -> Result<Self, io::Error> {
        Ok(if let Some(file_name) = file_name {
            Self::File(File::create(&file_name)?)
        } else {
            Self::Buffer(Vec::<u8>::new())
        })
    }
}

pub async fn download_with_progress(
    link: &str,
    file_name: Option<&str>,
) -> Result<DualWriter, String> {
    let mut dual_writer = DualWriter::new(file_name).or(Err("Failed to create file".to_owned()))?;

    let client = Client::builder()
        .gzip(true)
        .deflate(true)
        .build()
        .or(Err("Failed to create client".to_owned()))?;
    let resp = client
        .get(link)
        .send()
        .await
        .or(Err("Failed to connect server".to_owned()))?;
    let content_length = if let Some(got_content_length) = resp.content_length() {
        got_content_length
    } else {
        panic!("Could not retrive content length from server. {:?}", &resp);
    };

    let progress_bar = ProgressBar::new(content_length);
    progress_bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})").unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let bytes = item.unwrap();
        downloaded = cmp::min(downloaded + (bytes.len() as u64), content_length);
        dual_writer
            .write(bytes)
            .or(Err("Failed to write to file".to_owned()))?;

        progress_bar.set_position(downloaded);
    }

    Ok(dual_writer)
}
