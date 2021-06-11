use std::path::PathBuf;
use std::io::stdin;
use ansi_term::Color::{Red, Green};
use log::info;
use std::collections::HashMap;
use std::fs::OpenOptions;

use crate::manager::InstanceManager;
use crate::setup;
use crate::downloader::Downloader;
use crate::invoker::Invoker;
use crate::instance::InstanceType;
use crate::auth::User;


pub async fn setup(mut instance_manager: InstanceManager, user_path: PathBuf) {
    
    println!("Vanilla version: ");
    let mut vanilla_version = String::new();    

    // reading version from user input
    //stdout().flush();
    stdin().read_line(&mut vanilla_version).expect("Did not enter a correct string");
    if let Some('\n') = vanilla_version.chars().next_back() {
        vanilla_version.pop();
    }
    if let Some('\r') = vanilla_version.chars().next_back() {
        vanilla_version.pop();
    }    


    let body = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let manifest : serde_json::Value = serde_json::from_str(body.as_str()).unwrap();

    let versions = manifest["versions"].as_array().unwrap();

    let mut version_found = false;
    for version in versions {
        if version["id"].as_str().unwrap() == vanilla_version {
            version_found = true;
            // found version
            // time to install
            
            // creating new instance
            let instance = instance_manager.create_instance(format!("vanilla-{}", vanilla_version)).unwrap();
       
             

            let mut vanilla_manifest_path = instance.get_path();
            vanilla_manifest_path.push(format!("versions/{}/{}.json", vanilla_version, vanilla_version));


            let manifest_download_url = version["url"].as_str().unwrap().to_string();

            let mut version_download = HashMap::new();
            version_download.insert(vanilla_manifest_path.clone(), manifest_download_url);

            //download manifest
            Downloader::new(version_download)
                .process()
                .await
                .expect("Unable to download file");

            let version_manifest_file = OpenOptions::new()
                .read(true)
                .write(false)
                .open(vanilla_manifest_path.clone())
                .unwrap();

            let version_manifest_json : serde_json::Value = serde_json::from_reader(version_manifest_file).unwrap();

   
            let vmpc = vanilla_manifest_path.clone();
            let ip = instance.get_path();

            tokio::spawn(async move {
                setup::get_binaries(vmpc, ip).await;
            });
 


            // get libraries for vanilla
            let mut libpath = instance.get_path();
            libpath.push("libraries");

            // get a list of all downloads
            let mut downloads = setup::get_library_downloads(libpath.clone(), vanilla_manifest_path.clone()).await.unwrap();
            let mut narrator_path = instance.get_path();
            narrator_path.push("libraries/com/mojang/text2speech/1.10.3/text2speech-1.10.3.jar");
            let narrator_url =  "https://libraries.minecraft.net/com/mojang/text2speech/1.10.3/text2speech-1.10.3.jar".to_string();
            downloads.insert(narrator_path, narrator_url);


            // add client.jar to download
            let mut client_jarpath = instance.get_path();
            client_jarpath.push("client.jar");

            downloads.insert(client_jarpath, 
                             version_manifest_json["downloads"]["client"]["url"]
                             .as_str()
                             .unwrap()
                             .to_string()
            );

            downloads.extend(
                setup::get_asset_downloads(instance.get_path(), vanilla_manifest_path.clone())
                    .await
                    .unwrap()
            );


            Downloader::new(downloads)
                .process()
                .await
                .expect("Unable to finish download");


 

            let mut assets_path = instance.get_path();
            assets_path.push("assets");

            let mut classpaths = Vec::new();  
            let classes = setup::get_cp_from_version(PathBuf::from("libraries"), vec![vanilla_manifest_path]);

            for class in classes {
                classpaths.push(class.1);
            }
            classpaths.push(PathBuf::from("client.jar"));
           
            let asset_index =  version_manifest_json["assetIndex"]["id"].as_str().unwrap();
            let main_class =  version_manifest_json["mainClass"].as_str().unwrap();

            let user = User::from(user_path.clone());

            // using relative binpath
            let mut invoker = Invoker::new(
                "java ".to_string(),
                PathBuf::from("./bin"),
                classpaths,
                format!(" --assetsDir ./assets --assetIndex {} --gameDir . --version  {}  --versionType release --userType mojang", asset_index, vanilla_version),
                main_class.to_string(),
                instance.name(),
                InstanceType::Vanilla,
                user.name,
                user.token,
            );

            let mut invoker_file_path = instance.get_path();
            invoker_file_path.push("sml_invoker.json");

            invoker.export_as_json(invoker_file_path);
            break;
        }
    }

    if !version_found {
        println!("{}", Red.paint("Version not found. Exiting..."));
        std::process::exit(0);
    }else{
        info!("{}", Green.paint("Setup is complete!"));
    }

}
