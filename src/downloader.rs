use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;

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
        File::create(fp.clone()).expect("Error creating file");
        let _fs = self.get_file_size();

        let body = ureq::get(self.url.as_str()).call().unwrap();
        let total_size = body
            .header("Content-length")
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();

        let mut reader = body.into_reader();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(fp.clone())
            .unwrap();

        io::copy(&mut reader, &mut file).expect("Error writing to file");

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
.progress_chars("=> "));

        pb.finish_with_message("Finished download");

        Ok(())
    }
}
