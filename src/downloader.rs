//use crypto::{digest::Digest, sha1::Sha1};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Write};
use std::path::PathBuf;
use std::{fs, time::Duration};
use std::collections::HashMap;
//use reqwest::Client;

extern crate crypto;

use log::info;


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
            futures::stream::iter(downloads).map(|(path, url)|{
                let client = self.client.clone();
                tokio::spawn(async move {
                    let resp = client.get(Url::parse(url.as_str()).unwrap()).send().await.unwrap();

                    info!("Downloading {}", url);
                    (path, resp.bytes().await)
                })
            }).buffer_unordered(8);
        
        fetches.for_each(|r| async move {
            match r {
                Ok((path, bytes)) => {
                    match bytes {
                        Ok(bytes) => {
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
                                .open(path.clone())
                                .unwrap();

                            file.write_all(&bytes[..]).expect("Error writing to file while downloading");

                            info!("Finished download. Saved to {}",path.display());
                        },
                        Err(e) => panic!("{}",e)
                    }
                },
                Err(e) => panic!("{}", e)
                
            }
            
            
        }).await;

        //fetches.await;

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


}
