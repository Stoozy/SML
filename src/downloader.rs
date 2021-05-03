use ansi_term::Color::*;
use crypto::{digest::Digest, sha1::Sha1};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs, time::Duration};

extern crate crypto;

use futures::future;

#[derive(Clone)]
pub struct Downloader {
    client: reqwest::Client,
    url: Option<String>,
    file_path: Option<PathBuf>,
    sha1: Option<String>,
}

impl Downloader {
    pub fn new() -> Downloader {
        let c = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        Downloader {
            client: c,
            url: None,
            file_path: None,
            sha1: None,
        }
    }

    pub fn set_url(&mut self, u: String) {
        self.url = Some(u);
    }

    pub fn set_path(&mut self, fp: PathBuf) {
        self.file_path = Some(fp);
    }

    pub fn set_sha1(&mut self, s: String) {
        self.sha1 = Some(s);
    }

    // Verify file integrity
    pub fn verify_sha1(&self) -> Option<bool> {
        let fp = self.file_path.clone().unwrap();
        if self.sha1.is_none() {
            return None;
        }

        let mut hasher = Sha1::new();
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(fp.clone())
            .unwrap();
        let mut file_data = Vec::new();
        match file.read_to_end(&mut file_data) {
            Ok(_) => (),
            Err(e) => {
                println!("{}: {}", Red.paint("Unable to read file data"), e);
            }
        }

        hasher.input(&file_data);
        let hex = hasher.result_str();

        return Some(hex == self.sha1.clone().unwrap());
    }

    pub async fn download(&mut self, show_bar: bool) -> Result< (), Box<dyn std::error::Error>> {
        let fp = self.file_path.clone().unwrap();
        if show_bar {
            println!("Downloading {}", fp.display());
            println!("URL: {}", self.url.clone().unwrap());
        }

        // if parent dir doesn't exist
        // recursively create all of them
        let parent = fp.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Couldn't create parent directories");
        }

        // create file
        File::create(self.file_path.clone().unwrap()).expect("Error creating file");

        let data = self
            .client
            .get(self.url.clone().unwrap().as_str())
			.send()
			.await?
			.bytes()
			.await?;
		// keep retrying until some data is available
		//if data.is_empty() {
		//	return self.download(show_bar).await;
		//}

		let mut file = OpenOptions::new()
			.read(true)
			.write(true)
			.open(fp.clone())
			.unwrap();

		/*
		simply write to file
		*/
		file.write_all(&data).unwrap();

		Ok(())
    }

    //pub fn download_verified(&mut self) {
    //    self.download(true).expect("Couldn't download assets");

    //    match self.verify_sha1() {
    //        Some(b) => {
    //            if b {
    //                println!("{}", Green.paint("File Verified!"));
    //            } else {
    //                println!("{}", Yellow.paint("File not verfied. Re-downloading..."));
    //                self.download_verified();
    //            }
    //        }
    //        None => {
    //            println!("{}", Yellow.paint("File not verfied. Re-downloading..."));
    //            self.download_verified();
    //        }
    //    }
    //}
}
