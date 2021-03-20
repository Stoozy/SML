use crypto::{digest::Digest, sha1::Sha1};
use reqwest;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{ Read, Write};
use std::path::PathBuf;
use ansi_term::Color::*;

extern crate crypto;

const CHUNK_SIZE: usize = 4096;

#[derive(Clone)]
pub struct Downloader {
    url: String,
    file_path: PathBuf,
    sha1: Option<String>,
}

impl Downloader {
    pub fn new(u: String, fp: PathBuf) -> Downloader {
        Downloader {
            url: u,
            file_path: fp,
            sha1: None
        }
    }

    pub fn add_sha1(&mut self, s : String) -> Downloader{
        self.sha1 = Some(s); 
        self.clone()
    }

    // Verify file integrity
    pub fn verify_sha1(&self) -> Option<bool> {
        if self.sha1.is_none() {
            return None;
        } 

        let mut hasher = Sha1::new();
        let mut file = OpenOptions::new()
                    .read(true)
                    .write(false)
                    .open(self.file_path.clone())
                    .unwrap();
        let mut file_data = Vec::new();
        match file.read_to_end(&mut file_data){
            Ok(_) => (),
            Err(e) => {
                println!("{}: {}", Red.paint("Unable to read file data"), e);
            }
        }

        hasher.input(&file_data);
        let hex = hasher.result_str();

        return Some(hex == self.sha1.clone().unwrap());
    }


    pub fn download(&mut self) -> Result<(), reqwest::Error> {
        let fp = self.file_path.clone();
        println!("Downloading {}", fp.display());

        // if parent dir doesn't exist
        // recursively create all of them
        let parent = self.file_path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Couldn't create parent directories");
        }

        // create file 
        File::create(fp.clone()).expect("Error creating file");

        match reqwest::blocking::get(self.url.as_str())
                                    .unwrap()
                                    .bytes()
        {
            Ok(data)  => {

                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(fp.clone())
                    .unwrap();


                let pb = ProgressBar::new(data.len() as u64);

                pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
.progress_chars("=> "));


                for i in 0..data.len() / CHUNK_SIZE {
                    if i != (data.len() / CHUNK_SIZE) - 1 {
                        file.write_all(&data[i * CHUNK_SIZE..(i + 1) * CHUNK_SIZE])
                                .expect("Error writing to file");
                    } else {
                        // write the entire last part
                        file.write_all(&data[i * CHUNK_SIZE..])
                                .expect("Error writing to file");
                    }

                        pb.set_position(i as u64);
                }

                pb.finish_with_message("Finished download");
            },
            Err(e) => {
                println!("Download failed");
                return Err(e);
            }
            
        }

        Ok(())
    }
}
