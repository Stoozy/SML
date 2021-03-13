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

    pub fn download(&mut self) -> Result<(), ureq::Error> {
        let fp = self.file_path.clone();
        let mut body = ureq::get(self.url.as_str()).call().unwrap().into_reader();

        File::create(fp.clone()).expect("Error creating file");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(fp.clone())
            .unwrap();

        io::copy(&mut body, &mut file).expect("Error writing to file");

        Ok(())
    }
}
