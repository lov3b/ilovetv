use std::{
    error, fmt,
    fs::{File, OpenOptions},
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

    pub fn len(&self) -> u64 {
        match self {
            DualWriter::File(f) => f.metadata().and_then(|x| Ok(x.len())).unwrap_or_else(|e| {
                println!("Could not get metadata from file {:?}", e);
                0
            }),
            DualWriter::Buffer(buf) => buf.len() as u64,
        }
    }
}

impl TryFrom<Option<&str>> for DualWriter {
    type Error = io::Error;

    fn try_from(file_name: Option<&str>) -> Result<Self, Self::Error> {
        Ok(if let Some(file_name) = file_name {
            let file = match OpenOptions::new().append(true).open(&file_name) {
                Ok(f) => f,
                Err(e) => {
                    let e = e as io::Error;
                    if e.kind() == io::ErrorKind::NotFound {
                        File::create(&file_name)?
                    } else {
                        return Err(e);
                    }
                }
            };
            Self::File(file)
        } else {
            Self::Buffer(Vec::<u8>::new())
        })
    }
}

impl TryInto<String> for DualWriter {
    type Error = String;

    fn try_into(self) -> Result<String, Self::Error> {
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
}

pub async fn download_with_progress(
    link: &str,
    file_name: Option<&str>,
) -> Result<DualWriter, Box<dyn error::Error>> {
    let mut dual_writer: DualWriter = file_name.try_into()?;

    let client = Client::builder().gzip(true).deflate(true).build()?;
    let builder = client
        .get(link)
        .header("Range", format!("bytes={}-", dual_writer.len()));

    let resp = builder.send().await?;

    let content_length = resp.content_length().unwrap_or_default();
    if content_length == 0 {
        println!("File was already downloaded");
        return Ok(dual_writer);
    }

    let progress_bar = ProgressBar::new(content_length);
    progress_bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})").unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| write!(w, "{:.1}m", state.eta().as_secs_f64() / 60.0 ).unwrap())
        .progress_chars("#>-"));

    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let bytes = item.unwrap();
        downloaded = cmp::min(downloaded + (bytes.len() as u64), content_length);
        dual_writer.write(bytes)?;

        progress_bar.set_position(downloaded);
    }

    Ok(dual_writer)
}
