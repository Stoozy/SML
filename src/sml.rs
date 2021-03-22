use crate::{cf::CFFile, downloader::Downloader};
use crate::ima::Instance;
use serde_json::*;
use std::{fs::{File, OpenOptions}, io::{self, BufReader}};
use std::{fs, path::PathBuf};
use zip::ZipArchive;
use serde::{ Serialize, Deserialize};
use ansi_term::Color::*;


mod util;

const CHUNK_SIZE: usize = 8192;

// needed for serde json serialization
#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub token: String,
}

pub fn get_modslist(chosen_proj: CFFile, instance: Instance) {
    let download_url = chosen_proj.get_download_url();
    let mut download_path = instance.get_path();
    download_path.push("mods/");
    if !download_path.exists() {
        fs::create_dir(download_path.clone()).expect("Error creating mods folder");
    }
    download_path.push(chosen_proj.name.clone());

    println!("Got download url {}", download_url);
    println!("Got download path {}", download_path.display());

    let mut downloader = Downloader::new();
    downloader.set_url(download_url);
    downloader.set_path(download_path.clone());
    downloader.download().expect("Error downloading modslist");

    let mut mod_dirpath = instance.get_path().clone();
    mod_dirpath.push("mods/");

    // extract zip
    let modpack_zip = fs::File::open(download_path.clone()).expect("Couldn't open modslist");
    println!("Downloaded mods list");

    println!("Extracting mods list");
    let mut zip = ZipArchive::new(modpack_zip).unwrap();
    let mut extract_path = download_path.clone();
    extract_path.pop();

    zip.extract(extract_path)
        .expect("Error extracting mods list");

    fs::remove_file(download_path.clone()).expect("Error deleting stage zip file");
}


pub fn get_cp_from_version(libpath: PathBuf, version_paths : Vec<PathBuf>) -> Vec<(String, PathBuf)> {
    let mut retvec = Vec::new();

    
    for version_fpath in version_paths {
        let file = File::open(version_fpath).unwrap();
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let u : serde_json::Value = serde_json::from_reader(reader).unwrap();

        let libraries = u["libraries"].as_array().unwrap();

        for lib in libraries {
            let artifact : Vec<&str> = lib["name"]
                            .as_str()
                            .unwrap()
                            .split(":")
                            .collect();

            let name = artifact[1];
            let version = artifact[2];

            let full_name = format!("{}:{}:{}", artifact[0], artifact[1], artifact[2]);

            let mut path = libpath.clone();
            path.push( match lib["downloads"]["artifact"]["path"].as_str(){
                Some(val) =>  val,
                None => {
                    println!("Couldn't get library path, skipping");
                    ""
                },
            });

            // this excludes forge or any other invalid lib for the check
            if lib["downloads"]["artifact"]["url"].as_str().is_none()  {
                retvec.push((full_name, path));
            }else{

                let mut found_version = "";
                let found_index = retvec.iter().position(|v|{
                    let a = &v.0;
                    let n : Vec<&str> = a.split(":").collect();
                    found_version = n[2];
                    name == n[1]
                });



                // make some checks for duplicate library
                if !found_index.is_none() {

                    if util::is_greater_version(version, found_version) {
                        // prev version is old
                        // remove it and put new one
                        retvec.remove(found_index.unwrap());
                        retvec.push((full_name, path));
                    }
                    // if prev entry has greater version, 
                    // then don't push anything
                
                }else{
                    // no duplicates found, may push
                    retvec.push((full_name, path));
                }

            }
        }

    }

    retvec
}

pub fn get_libraries(libpath: PathBuf, manifests: Vec<PathBuf>) -> Result<()> {
    for manifest in manifests{

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(manifest.clone())
        .unwrap();


        let json: serde_json::Value = serde_json::from_reader(file).unwrap();
        let libraries = json["libraries"].as_array().expect("Error getting libraries.");
        let mut downloader = Downloader::new();


        for lib in libraries.iter(){

            let artifact_path = match lib["downloads"]["artifact"]["path"].as_str(){
                Some(val) => val,
                None => {
                    println!("Error getting library, skipping ...");
                    break;
                }
            };

            let mut path = libpath.clone(); 
            path.push(artifact_path);

            let download_url = match lib["downloads"]["artifact"]["url"].as_str(){
                Some(val) => val,
                None => {
                    println!("{}:{}  {}", 
                            Red.paint("Library url is missing"),
                            path.display(),
                            Yellow.paint("skipping ..."));
                    break;
                }

            };

            let artifact_sha1 = match lib["downloads"]["artifact"]["sha1"].as_str(){
                Some(hash) => hash,
                None => {
                    println!("No hash found , skipping ...");
                    break;
                }
            };

            
            // only download if url is valid
            if download_url != "" {
                downloader.set_url(download_url.to_string());
                downloader.set_path(path);
                downloader.set_sha1(artifact_sha1.to_string());
                match downloader.download() {
                    Ok(_) => {
                        //match downloader.verify_sha1(){
                        //    Some(mut is_verified) => {
                        //        while !is_verified {
                        //            println!("Invalid hash: {}",  Yellow.paint("Retrying download..."));
                        //            println!("URL: {}", download_url);
                        //            downloader.download().unwrap();
                        //            is_verified = downloader.verify_sha1().unwrap();

                        //        }

                        //        println!("{} sha1:{}", Green.paint("File verified!"),  artifact_sha1);
                        //    },
                        //    None => {
                        //        println!("{}", Red.paint("Failed to verify file"));
                        //    }
                        //};
                    },
                    Err(_) => {
                        println!("{} {}", Red.paint("Failed to download"), artifact_path);
                        continue
                    }
                };
            }
        }

    }

    Ok(())
}

pub fn get_assets(game_path: PathBuf, version_path: PathBuf) -> Result<()> {
    let version_file = File::open(version_path).unwrap();
    let version : serde_json::Value = serde_json::from_reader(version_file).unwrap();

    let url = match version["assetIndex"]["url"].as_str(){
        Some(val) => val,
        None => {
            println!("Error getting assetIndex. Skipping.");
            return Ok(());
        }
    };
    let assets_json : serde_json::Value = ureq::get(url)
                            .call()
                            .unwrap()
                            .into_json()
                            .unwrap();

    let asset_objects = assets_json["objects"].as_object().unwrap();

    for object in asset_objects{
        let hash = object.1["hash"].as_str().unwrap();
        let first_two = &hash[0..2];

        let mut save_path = game_path.clone();
        save_path.push("assets/objects/");
        save_path.push(first_two);
        save_path.push(hash);

        let download_url = format!("http://resources.download.minecraft.net/{}/{}", first_two, hash);
        println!("Got url: {}", download_url);
        let mut downloader = Downloader::new();
        downloader.set_path(save_path);
        downloader.set_url(download_url);
        downloader.download().expect("Couldn't download assets");
    }
    
    Ok(())
}

pub fn get_mods(mods_path: PathBuf){
    let mut mods_manifest_path = mods_path.clone();
    mods_manifest_path.push("manifest.json");

    let manifest_reader = File::open(mods_manifest_path).unwrap();
    let manifest : serde_json::Value = serde_json::from_reader(manifest_reader)
                                        .expect("Couldn't get mod manifest");


    let mods = manifest["files"].as_array().unwrap();
    
    for m in mods {
        let proj_id = m["projectID"].as_u64().unwrap();
        let file_id = m["fileID"].as_u64().unwrap();

        let mod_json : serde_json::Value = ureq::get(format!("https://api.cfwidget.com/{}", proj_id).as_str())
                            .call()
                            .unwrap()
                            .into_json()
                            .unwrap();
        let modfiles = match mod_json["files"].as_array(){
            Some(val) => val,
            None => {
                println!("Could not parse files list");
                continue;
            } 
        };

        for modfile in modfiles {
            // found right mod file now download it
            if modfile["id"].as_u64().unwrap() == file_id {

               let cf_file = CFFile{
                   id: file_id,
                   display: modfile["display"].as_str().unwrap().to_string(),
                   name: modfile["name"].as_str().unwrap().to_string(),
                   ftype: modfile["type"].as_str().unwrap().to_string(),
                   version: modfile["version"].as_str().unwrap().to_string()};

               let download_url = cf_file.get_download_url();
               let mut download_path = mods_path.clone();
               download_path.push(cf_file.name);
    
               // if one mod errors, then continue with the rest
               // TODO: Track and display broken files 
               let mut downloader = Downloader::new();
               downloader.set_path(download_path);
               downloader.set_url(download_url);
               match downloader.download(){
                   Ok(_) => continue,
                   Err(e) => panic!("{}", e),

               }
             }
        }

    }
}



pub fn get_fv_from_mcv(mcv: String) -> String {
    let versions_url = "https://files.minecraftforge.net/maven/net/minecraftforge/forge/promotions_slim.json"; 
    let versions_json : serde_json::Value = ureq::get(versions_url)
                                            .call()
                                            .unwrap()
                                            .into_json()
                                            .unwrap();
    let key = format!("{}-latest", mcv);
    versions_json["promos"][key]
        .as_str()
        .expect("Couldn't get forge versions list")
        .to_string()

}


pub fn get_forge_args(json: serde_json::Value) -> Option<String>{
    let mut retstr = String::new();
    // get args here
    let args = json["arguments"]["game"].as_array().unwrap();
    for arg in args{
        retstr.push(' ');
        retstr.push_str(arg.as_str().unwrap());
    } 

    Some(retstr)
}


