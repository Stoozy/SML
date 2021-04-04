use crypto::{digest::Digest, sha1::Sha1};
// use reqwest::{ClientBuilder, blocking::Client};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, time::Duration};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{ stdout, Read, Write};
use std::path::PathBuf;
use ansi_term::Color::*;
use curl::easy::Easy;

extern crate crypto;

const CHUNK_SIZE: usize = 4096;

#[derive(Clone)]
pub struct Downloader {
    client : reqwest::blocking::Client,
    url: Option<String>,
    file_path: Option<PathBuf>,
    sha1: Option<String>,
}

impl Downloader {
    pub fn new () -> Downloader {
        let c = reqwest::blocking::ClientBuilder::new()
                            .timeout(Duration::from_secs(300))
                            .build()
                            .unwrap();
        Downloader {client: c, url: None, file_path: None, sha1: None}
    }

    pub fn set_url(&mut self, u: String) -> () {
        self.url = Some(u);
    }

    pub fn set_path(&mut self, fp: PathBuf)-> () {
        self.file_path = Some(fp);
    }

    pub fn set_sha1(&mut self, s : String) -> Downloader{
        self.sha1 = Some(s); 
        self.clone()
    }

    // Verify file integrity
    pub fn verify_sha1(& self) -> Option<bool> {
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


    pub fn download(&mut self, show_bar: bool) -> Result<(), reqwest::Error> {
        let fp = self.file_path.clone().unwrap();
        if show_bar{
            println!("Downloading {}", fp.display());
        }
        
        // if parent dir doesn't exist
        // recursively create all of them
        let parent = fp.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Couldn't create parent directories");
        }

        // create file 
        File::create(self.file_path.clone().unwrap()).expect("Error creating file");

        //let mut easy = Easy::new();
        //easy.url(self.url.clone().unwrap().as_str()).unwrap();
        //let fp = self.file_path.clone();
        //easy.write_function(move |data| {
        //    let mut file = OpenOptions::new()
        //            .read(true)
        //            .write(true)
        //            .open(fp.clone().unwrap().clone())
        //            .unwrap();

        //    file.write(data).unwrap();
        //    Ok(data.len())
        //}).unwrap();
        //easy.perform().unwrap();

        //println!("{}", Green.paint("Download finished!"));
        //println!("{}", easy.response_code().unwrap());

        match self.client.get(self.url.clone().unwrap().as_str())
                                    .send()
                                    .unwrap()
                                    .bytes()
        {
            Ok(data)  => {
                // keep retrying until some data is available
                if data.len() == 0 { return self.download(show_bar);}

                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(fp.clone())
                    .unwrap();


                if show_bar {
                    // with progressbar
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


                }else{
                    // no progress bar
                    for i in 0..data.len() / CHUNK_SIZE {
                        if i != (data.len() / CHUNK_SIZE) - 1 {
                            file.write_all(&data[i * CHUNK_SIZE..(i + 1) * CHUNK_SIZE])
                                    .expect("Error writing to file");
                        } else {
                            // write the entire last part
                            file.write_all(&data[i * CHUNK_SIZE..])
                                    .expect("Error writing to file");
                        }

                    }
                }


            },
            Err(_e) => {
                println!("Download failed"); 
                return self.download(show_bar); // keep trying when download fails
            }
            
        }

        Ok(())
    }


    pub fn download_verified(&mut self)  {
        self.download(true).expect("Couldn't download assets");
        
        match self.verify_sha1(){
            Some(b) => {
                if b {
                    println!("{}", Green.paint("File Verified!"));
                }else{
                    println!("{}", Yellow.paint("File not verfied. Re-downloading..."));
                    self.download_verified();
                }
            },
            None => {
                    println!("{}", Yellow.paint("File not verfied. Re-downloading..."));
                    self.download_verified();
            }
        }

    }



}
