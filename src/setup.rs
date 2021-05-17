//use crate::auth;
use crate::forge;
use crate::ima::{Instance, InstanceManager};
use crate::invoker::Invoker;
use crate::util;

use crate::{
    cf::{CFFile, CFProject},
    downloader::Downloader,
};
use ansi_term::Color::*;
//use serde_json::*;

use log::{info, warn};
use crate::auth::User;

use std::{
    collections::HashMap,
    fs,
    fs::{File, OpenOptions},
    io::BufReader,
    path::PathBuf,
};

use zip::ZipArchive;

// needed for serde json serialization

pub async fn get_modslist(chosen_proj: CFFile, instance: Instance) {
    let download_url = chosen_proj.get_download_url();
    let mut download_path = instance.get_path();
    download_path.push("mods");
    if !download_path.exists() {
        fs::create_dir(download_path.clone()).expect("Error creating mods folder");
    }
    download_path.push(chosen_proj.name);
    let mut downloads = HashMap::new();
    downloads.insert(download_path.clone(), download_url);
    let downloader = Downloader::new(downloads);
    
    downloader.process().await.unwrap();

    let mut mod_dirpath = instance.get_path();
    mod_dirpath.push("mods");

    // extract zip
    let modpack_zip = fs::File::open(download_path.clone()).expect("Couldn't open modslist");
    println!("Downloaded mods list");

    println!("Extracting mods list");
    let mut zip = ZipArchive::new(modpack_zip).unwrap();
    let mut extract_path = download_path.clone();
    extract_path.pop();

    zip.extract(extract_path)
        .expect("Error extracting mods list");

    fs::remove_file(download_path).expect("Error deleting stage zip file");
}

pub fn get_cp_from_version(
    libpath: PathBuf,
    version_paths: Vec<PathBuf>,
) -> Vec<(String, PathBuf)> {
    let mut retvec = Vec::new();

    for version_fpath in version_paths {

        // add version jar to path

        let mut version_jarpath = version_fpath.clone();
        version_jarpath.set_extension("jar");

        if version_jarpath.exists() {
            retvec.push(("version_jar".to_string(), version_jarpath));
        }

        let file = File::open(version_fpath).unwrap();
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let u: serde_json::Value = serde_json::from_reader(reader).unwrap();

        let libraries = u["libraries"].as_array().unwrap();

        for lib in libraries {
            let artifact: Vec<&str> = lib["name"].as_str().unwrap().split(":").collect();

            let name = artifact[1];
            let version = artifact[2];

            let full_name = format!("{}:{}:{}", artifact[0], artifact[1], artifact[2]);

            let mut path = libpath.clone();
            path.push(match lib["downloads"]["artifact"]["path"].as_str() {
                Some(val) => val,
                None => {
                    dbg!(lib);
                    println!("Couldn't get library path for {}, skipping", lib["downloads"]["artifact"]["name"]);
                    ""
                }
            });

            // this excludes forge or any other invalid lib for the check
            // since they don't have urls
            if lib["downloads"]["artifact"]["url"].as_str().is_none() {
                retvec.push((full_name, path));
            } else {

                let mut found_version = Some(String::new());
                let found_index = retvec.iter().position(|v| {
                    let a = &v.0;
                    let n: Vec<&str> = a.split(":").collect();

                    if n.len() >= 3 {
                        found_version = Some(n[2].to_string());
                        name == n[1]
                    }else{
                        false
                    }

                });

                // make some checks for duplicate library
                if found_index.is_some() {
                    match found_version {
                        Some(other_version) => {
                            if util::geq_version(version, other_version.as_str()) {
                                // remove old version and keep new one
                                retvec.remove(found_index.unwrap());
                                retvec.push((full_name, path));
                            }
                        },
                        None => ()
                    }
                    
                    // if prev entry has greater version,
                    // then don't push anything
                } else {
                    // no duplicates found, may push
                    retvec.push((full_name, path));
                }
            }
        }
    }

    retvec
}

pub async fn get_library_downloads(
    libpath: PathBuf,
    manifest: PathBuf,
) -> Option<HashMap<PathBuf, String>> {

    let mut lib_downloads: HashMap<PathBuf, String> = HashMap::new();

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(manifest)
        .unwrap();

    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    let libraries = json["libraries"]
        .as_array()
        .expect("Error getting libraries.");

    for (_i, lib) in libraries.iter().enumerate() {
        let artifact_path = match lib["downloads"]["artifact"]["path"].as_str() {
            Some(val) => val,
            None => {
                // skipping empty paths
                break;
            }
        };

        let mut path = libpath.clone();
        path.push(artifact_path);

        let download_url = match lib["downloads"]["artifact"]["url"].as_str() {
            Some(val) => val,
            None => {
                // skipping on empty url
                break;
            }
        };

        // only download if url is valid
        if !download_url.is_empty() {
            lib_downloads.insert(path, download_url.to_string());
        }
    }

    Some(lib_downloads)
}

pub async fn get_asset_downloads(
    game_path: PathBuf,
    version_path: PathBuf,
) -> Option<HashMap<PathBuf, String>> {

    let mut asset_downloads: HashMap<PathBuf, String> = HashMap::new();

    let request_client = reqwest::Client::new();

    let version_file = File::open(version_path).unwrap();
    let version: serde_json::Value = serde_json::from_reader(version_file).unwrap();

    let url = version["assetIndex"]["url"].as_str()?;

    // download assetIndex json file
    let mut index_save_path = game_path.clone();
    index_save_path.push("assets");
    index_save_path.push("indexes");
    index_save_path.push(format!(
        "{}.json",
        version["assetIndex"]["id"].as_str().unwrap()
    ));

    asset_downloads.insert(index_save_path, url.to_string());

    let resp = request_client.get(
            reqwest::Url::parse(url).unwrap()
        ).send().await.unwrap();
    let assets_json : serde_json::Value = serde_json::from_str(resp.text().await.unwrap().as_str()).unwrap();

    let asset_objects = assets_json["objects"].as_object().unwrap();

    for (_i, object) in asset_objects.iter().enumerate() {
        let hash = object.1["hash"].as_str().unwrap();
        let first_two = &hash[0..2];

        let mut save_path = game_path.clone();
        save_path.push("assets");
        save_path.push("objects");
        save_path.push(first_two);
        save_path.push(hash);

        let download_url = format!(
            "http://resources.download.minecraft.net/{}/{}",
            first_two, hash
        );

        asset_downloads.insert(save_path, download_url);
    }

    Some(asset_downloads)
}

pub async fn get_mod_downloads(
    mc_version: String,
    mods_path: PathBuf,
) -> Option<HashMap<PathBuf, String>> {

    // create a reqwest client
    let request_client = reqwest::Client::new();

    let mut downloads_map: HashMap<PathBuf, String> = HashMap::new();

    let mut mods_manifest_path = mods_path.clone();
    mods_manifest_path.push("manifest.json");

    let manifest_reader = File::open(mods_manifest_path).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_reader(manifest_reader).expect("Couldn't get mod manifest");

    let mods = manifest["files"].as_array().unwrap();

    for m in mods {
        let proj_id = m["projectID"].as_u64().unwrap();
        let file_id = m["fileID"].as_u64().unwrap();

        //let mod_json: serde_json::Value =
            
        let resp = request_client.get(
            reqwest::Url::parse(format!("https://api.cfwidget.com/{}", proj_id).as_str()).unwrap()
        ).send().await.unwrap();
        let mod_json : serde_json::Value = serde_json::from_str(resp.text().await.unwrap().as_str()).unwrap();


        let for_versions = mod_json["versions"].as_array();

        // if versions key exists in the mod manifest json, use that
        if for_versions.is_some() {
            for version in for_versions.unwrap() {
                let v = version.as_str().unwrap();
                if v == mc_version {
                    // search the fileID here
                    let files = version.as_array().unwrap();
                    for file in files {
                        let cfid = file["id"].as_u64().unwrap();
                        if cfid == file_id {
                            let cf_file = CFFile {
                                id: file_id,
                                display: version["display"].as_str().unwrap().to_string(),
                                name: version["name"].as_str().unwrap().to_string(),
                                ftype: version["type"].as_str().unwrap().to_string(),
                                version: version["version"].as_str().unwrap().to_string(),
                            };

                            let download_url = cf_file.get_download_url();

                            let mut download_path = mods_path.clone();
                            download_path.push(cf_file.name);

                            // push to map instead of downloading here directly
                            downloads_map.insert(download_path, download_url);
                        }
                    }
                }
            }
        } else {
            let modfiles = match mod_json["files"].as_array() {
                Some(val) => val,
                None => {
                    dbg!(mod_json);
                    warn!("Could not parse files list");
                    continue;
                }
            };

            for (_i, modfile) in modfiles.iter().enumerate() {
                // found right mod file now download it
                if modfile["id"].as_u64().unwrap() == file_id {
                    let cf_file = CFFile {
                        id: file_id,
                        display: modfile["display"].as_str().unwrap().to_string(),
                        name: modfile["name"].as_str().unwrap().to_string(),
                        ftype: modfile["type"].as_str().unwrap().to_string(),
                        version: modfile["version"].as_str().unwrap().to_string(),
                    };

                    let download_url = cf_file.get_download_url();
                    let mut download_path = mods_path.clone();
                    download_path.push(cf_file.name);

                    downloads_map.insert(download_path, download_url);
                }
            }
        }
    }

    Some(downloads_map)
}

pub async fn get_binaries(version_path: PathBuf, instance_path: PathBuf) {
    let manifest_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(version_path)
        .expect("Couldn't open version path");

    let json: serde_json::Value =
        serde_json::from_reader(manifest_file).expect("Could not parse version file");

    let libs = json["libraries"].as_array().unwrap();
    let os = std::env::consts::OS;

    let mut jarpaths: Vec<PathBuf> = Vec::new();

    let mut download_map : HashMap<PathBuf, String> = HashMap::new();
    // Download jars
    for lib in libs {
        if !lib["downloads"]["classifiers"].is_null() {
            let natives_id: &str = match os {
                "windows" => "natives-windows",
                "macos" => "natives-macos",
                "linux" => "natives-linux",
                _ => "",
            };

            if natives_id != "" {
                let url = match lib["downloads"]["classifiers"][natives_id]["url"].as_str() {
                    Some(s) => s,
                    None => break,
                };

                let path = match lib["downloads"]["classifiers"][natives_id]["path"].as_str() {
                    Some(s) => s,
                    None => break,
                };


                let mut fullpath = instance_path.clone();
                fullpath.push("libraries");
                fullpath.push(path);

                download_map.insert(fullpath, url.to_string());
                jarpaths.push(PathBuf::from(path));

            } else {
                panic!("Couldn't detect OS");
            }
        }
    }

    // Download binaries
    Downloader::new(download_map)
        .process()
        .await
        .expect("Unable to download file");

    // Extract jars
    for jarpath in jarpaths {
        let mut fullpath = instance_path.clone();
        fullpath.push("libraries/");
        fs::create_dir_all(fullpath.clone()).expect("Couldn't create binary directory");
        fullpath.push(jarpath);

        let jarfile = OpenOptions::new()
            .read(true)
            .write(true)
            .open(fullpath)
            .unwrap();

        let mut za = ZipArchive::new(jarfile).unwrap();
        let mut binpath = instance_path.clone();
        binpath.push("bin/");
        fs::create_dir_all(binpath.clone()).expect("Couldn't create binary directory");
        za.extract(binpath).expect("Couldn't extract binary.");
    }
}


pub async fn forge_setup(mut ima: InstanceManager, id: u64, user_path: PathBuf) {
    let mut proj = CFProject::new(id, "https://api.cfwidget.com/".to_string());

    let choice = proj.get_choice().await.unwrap();

    let name = proj
        .files
        .iter()
        .nth(choice)
        .unwrap()
        .clone()
        .name()
        .chars()
        .map(|c| match c {
            ' ' => '-',
            _ => c,
        })
        .collect();

    let instance = ima.create_instance(name).expect("Error creating instance");

    get_modslist(proj.files[choice].clone(), instance.clone()).await;

    let mut manifest_path = instance.get_path();
    manifest_path.push("mods");
    manifest_path.push("manifest.json");

    let manifest_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(manifest_path)
        .expect("Unable to open manifest file");

    let manifest_json: serde_json::Value =
        serde_json::from_reader(manifest_file).expect("Manifest contains invalid json");

    let modloader = manifest_json["minecraft"]["modLoaders"][0]["id"]
        .as_str()
        .unwrap();

    // format is like `forge-${version}`
    let modloader_split: Vec<&str> = modloader.split('-').collect();

    if modloader_split[0] != "forge" {
        println!("{}", Red.paint("This is not a forge modpack. Quitting..."));
        return;
    }

    let mcv = manifest_json["minecraft"]["version"].as_str().unwrap();
    let fv = modloader_split[1];

    let mc_forge_version = format!("{}-{}", mcv, fv);

    let mut launcher_profiles_path = instance.get_path();
    launcher_profiles_path.push("launcher_profiles.json");
    fs::write(launcher_profiles_path, "{\"profiles\": {} }")
        .expect("Error writing to launcher profiles");

    forge::download_installer(instance.get_path(), mc_forge_version.clone()).await;

    // forge headless installer for 1.13.2+
    // no need for headless installer here as forge
    // supports -installClient pre 1.13.2
    let is_pre_13 = !util::geq_version(mcv, "1.13.2");

    if is_pre_13 {
        let installer_cp = match cfg!(windows) {
            true => format!("forge-{}-installer.jar",mc_forge_version),
            false => format!("forge-{}-installer.jar",mc_forge_version)
        };

        forge::run_forge_installation(instance.get_path(), installer_cp, false);
    }else{

        info!("POST 1.13.2");
        forge::download_headless_installer(instance.get_path()).await;
        let installer_cp = match cfg!(windows) {
            true => format!("forge-{}-installer.jar;forge-installer-headless-1.0.1.jar",mc_forge_version),
            false =>  format!("forge-{}-installer.jar:forge-installer-headless-1.0.1.jar",mc_forge_version)
        };

        forge::run_forge_installation(instance.get_path(), installer_cp, true);
    }

    let mut mods_path = instance.get_path();
    mods_path.push("mods");

    let mut libpath = instance.get_path();
    libpath.push("libraries");

    let mut binpath = instance.get_path();
    binpath.push("bin");

    let mut assets_path = instance.get_path();
    assets_path.push("assets");

    let mut forge_version_path = instance.get_path();
    forge_version_path.push(format!(
        "versions/{}-forge-{}/{}-forge-{}.json",
        mcv, fv, mcv, fv
    ));

    let mut vanilla_version_path = instance.get_path();
    vanilla_version_path.push(format!("versions/{}/{}.json", mcv, mcv));

    let version_paths = vec![vanilla_version_path.clone(), forge_version_path.clone()];

    let vvpc = vanilla_version_path.clone();
    let ip = instance.get_path();
    tokio::spawn(async move {
        get_binaries(vvpc, ip).await;
    });
    
    
    // get libraries for both vanilla and forge
    let vvpc = vanilla_version_path.clone();
    let fvpc = forge_version_path.clone();

    // get a list of all downloads
    let mut downloads = get_library_downloads(libpath.clone(), vvpc).await.unwrap();
    downloads.extend(get_library_downloads(libpath.clone(), fvpc).await.unwrap());

    let mpc = mods_path.clone();
    let mcvc = mcv.to_owned();
    downloads.extend(get_mod_downloads(mcvc, mpc).await.unwrap());
    downloads.extend(
        get_asset_downloads(instance.get_path(), vanilla_version_path)
            .await
            .unwrap()
    );

    Downloader::new(downloads)
        .process()
        .await
        .expect("Unable to finish download");

    let mut overrides_path = mods_path;
    overrides_path.push("overrides");

    util::copy_overrides(instance.get_path(), overrides_path);

    let user = User::from(user_path);

    let forge_json_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(forge_version_path)
        .expect("Couldn't open forge version file");

    let forge_json: serde_json::Value =
        serde_json::from_reader(forge_json_file).expect("Unable to parse forge json file");

    let mut vanilla_version_path = instance.get_path();
    vanilla_version_path.push(format!("versions/{}/{}.json", mcv, mcv));
    
    let version_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(vanilla_version_path)
        .unwrap();
    let version : serde_json::Value = serde_json::from_reader(version_file).expect("Unable to parse version file");
    let asset_index =  version["assetIndex"]["id"].as_str().unwrap();

    let main_class = forge_json["mainClass"]
        .as_str()
        .expect("Couldn't get main class");

    let forge_args = util::get_forge_args(forge_json.clone(), is_pre_13);

    let classes = get_cp_from_version(PathBuf::from("libraries"), version_paths);
    let mut classpaths: Vec<PathBuf> = Vec::new();

    for class in classes {
        classpaths.push(class.1);
    }

    if is_pre_13 {
        // Game args look like this
        //"minecraftArguments": "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --tweakClass net.minecraftforge.fml.common.launcher.FMLTweaker --versionType Forge", 

        let mut args = forge_json["minecraftArguments"].as_str().unwrap().to_string();

        // build game args 
        args = args.replace("${auth_player_name}", user.name.as_str());
        args = args.replace("${version_name}", proj.files[choice].version.as_str());
        args = args.replace("${game_directory}", instance.get_path().to_str().unwrap());
        args = args.replace("${assets_root}", assets_path.to_str().unwrap());
        args = args.replace("${assets_index_name}", asset_index);
        args = args.replace("${auth_uuid}", user.id.as_str());
        args = args.replace("${auth_access_token}", user.token.as_str());
        args = args.replace("${user_type}", "mojang");

        let mut invoker = Invoker::new(
                "java".to_string(),
                binpath.clone(),
                classpaths.clone(),
                args,
                main_class.to_string()
            );

        let mut invoker_file_path = instance.get_path();
        invoker_file_path.push("sml_invoker.json");

        invoker.gen_invocation();
        invoker.export_as_json(invoker_file_path);

    }else{

        // POST 1.13.2

        let mut invoker = Invoker::new(
                "java ".to_string(),
                binpath,
                classpaths,
                format!("{} --assetsDir {} --assetIndex {} --gameDir {} --version  {} --username {} --accessToken {} --versionType release --userType mojang",
				forge_args.unwrap(), assets_path.display(), asset_index, instance.get_path().display(), proj.files[choice].version, user.name, user.token),
                main_class.to_string()
                );

        let mut invoker_file_path = instance.get_path();
        invoker_file_path.push("sml_invoker.json");

        invoker.gen_invocation();
        invoker.export_as_json(invoker_file_path);

    }

    info!("{}", Green.paint("Setup is complete!"));
}
