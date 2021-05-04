use ansi_term::Color::*;
use crypto::{digest::Digest, sha1::Sha1};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs, time::Duration};
use std::collections::HashMap;
extern crate crypto;

use reqwest::Url;
use futures::StreamExt;

#[derive(Clone)]
pub struct Downloader {
    client: reqwest::Client,
    queue: HashMap<PathBuf, String>,
}

impl Downloader {
    pub fn new(map: HashMap<PathBuf, String>) -> Downloader {
        let c = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        Downloader {
            client: c,
            queue: map
        }
    }

    pub async fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        let downloads = self.queue.clone();
        let fetches = 
            futures::stream::iter(
                downloads.into_iter().map(|(path, url)| 
                    tokio::spawn(async move{

                    match reqwest::get(Url::parse(url.as_str()).unwrap()).await{
                            Ok(resp) => {
                                match resp.bytes().await {

                                    Ok(bytes) => {
                                        // if parent dir doesn't exist
                                        // recursively create all of them

                                        if path.is_dir(){
                                            println!("Path is a directory, cannot download, {}", path.display());
                                            println!("Url: {}", url);

                                        }else{
                                            let parent = path.parent().unwrap();
                                            fs::create_dir_all(parent).expect("Couldn't create parent directories");

                                            if !path.exists(){
                                                File::create(path.clone())
                                                    .unwrap();
                                            }
                                            // create file
                                            let mut file = OpenOptions::new()
                                                .write(true)
                                                .read(true)
                                                .open(path)
                                                .unwrap();

                                            file.write_all(&bytes[..]).expect("Error writing to file while downloading");
                                            println!("Download complete")

                                        }

                                                                            },
                                    Err(e) => panic!("{}", e)
                                };
                            },
                            Err(e) => panic!("{}", e)
                    };

                            //dbg!(url);
                            //dbg!(path);
                    }))
            ).buffer_unordered(8).collect::<Vec<_>>();
        fetches.await;


        //downloads.into_iter()
        //       .map(|(path, url)|{
        //        async move {

        //            println!("Downloading {}", path.display());
        //            println!("URL: {}", url.clone());

        //            // if parent dir doesn't exist
        //            // recursively create all of them
        //            let parent = path.parent().unwrap();
        //            if !parent.exists() {
        //                fs::create_dir_all(parent).expect("Couldn't create parent directories");
        //            }

        //            // create file
        //            File::create(path).expect("Error creating file");

        //            match reqwest::get(Url::parse(url.as_str()).unwrap())
        //                .await
        //            {
        //                Ok(resp) => {
        //                    return Ok(()):
        //                },
        //                Err(e) =>{
        //                    panic!("{}",e);
        //                }
        //            }

        //        //let data = self
        //        //    .client
        //        //    .get(self.url.clone().unwrap().as_str())
        //        //    .await
        //        //    .send()
        //        //    .await?
        //        //    .bytes()
        //        //    .await?;
    
        //        }

        //    })
        //).buffer_unordered(8);
        Ok(())
    }

    // Verify file integrity
    //pub fn verify_sha1(&self) -> Option<bool> {
    //    let fp = self.file_path.clone().unwrap();
    //    if self.sha1.is_none() {
    //        return None;
    //    }

    //    let mut hasher = Sha1::new();
    //    let mut file = OpenOptions::new()
    //        .read(true)
    //        .write(false)
    //        .open(fp.clone())
    //        .unwrap();
    //    let mut file_data = Vec::new();
    //    match file.read_to_end(&mut file_data) {
    //        Ok(_) => (),
    //        Err(e) => {
    //            println!("{}: {}", Red.paint("Unable to read file data"), e);
    //        }
    //    }

    //    hasher.input(&file_data);
    //    let hex = hasher.result_str();

    //    return Some(hex == self.sha1.clone().unwrap());
    //}



    //pub async fn download(&mut self, show_bar: bool) -> Result<(), Box<dyn std::error::Error>> {
    //    let fp = self.file_path.clone().unwrap();
    //    if show_bar {
    //        println!("Downloading {}", fp.display());
    //        println!("URL: {}", self.url.clone().unwrap());
    //    }

    //    // if parent dir doesn't exist
    //    // recursively create all of them
    //    let parent = fp.parent().unwrap();
    //    if !parent.exists() {
    //        fs::create_dir_all(parent).expect("Couldn't create parent directories");
    //    }

    //    // create file
    //    File::create(self.file_path.clone().unwrap()).expect("Error creating file");

    //    let data = self
    //        .client
    //        .get(self.url.clone().unwrap().as_str())
    //        .send()
    //        .await?
    //        .bytes()
    //        .await?;
    //    // keep retrying until some data is available
    //    //if data.is_empty() {
    //    //	return self.download(show_bar).await;
    //    //}

    //    let mut file = OpenOptions::new()
    //        .read(true)
    //        .write(true)
    //        .open(fp.clone())
    //        .unwrap();

    //    /*
    //    simply write to file
    //    */
    //    file.write_all(&data).unwrap();

    //    Ok(())
    //}

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
