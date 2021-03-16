use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;


const CHUNK_SIZE: usize = 8192;

pub struct Downloader {
    url: String,
    file_path: PathBuf,
}

impl Downloader {
    pub fn new(u: String, fp: PathBuf) -> Downloader {
        Downloader {
            url: u,
            file_path: fp,
        }
    }

    pub fn get_file_size(&self) -> u64 {
        fs::metadata(self.file_path.clone())
            .expect("Error getting filesize")
            .len()
    }

    pub fn download(&mut self) -> Result<(), ureq::Error> {
        let fp = self.file_path.clone();
        println!("Downloading {}", fp.display());

        let parent = self.file_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent).expect("Couldn't create parent directories");
        }


        File::create(fp.clone()).expect("Error creating file");

        let body = match ureq::get(self.url.as_str()).call().unwrap();
        let mut reader = body.into_reader();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(fp.clone())
            .unwrap();

        let mut data : Vec<u8> =  Vec::new();
        reader.read_to_end(&mut data).expect("Error reading file");

        let pb = ProgressBar::new(data.len() as u64);

        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
.progress_chars("=> "));



        for i in 0..data.len() / CHUNK_SIZE {
            if i != (data.len() / CHUNK_SIZE) - 1 {
                file.write_all(&data[i * CHUNK_SIZE..(i + 1) * CHUNK_SIZE])
                        .expect("Error writing to file");
                pb.set_position(i as u64);
            } else {
                // write the entire last part
                file.write_all(&data[i * CHUNK_SIZE..])
                        .expect("Error writing to file");
                pb.set_position(i as u64);
            }
        }


        pb.finish_with_message("Finished download");

        Ok(())
    }
}
