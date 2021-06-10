use std::path::PathBuf;
use std::fs::{self, OpenOptions};
use log::info;
use ansi_term::Color::{Red, Green, Yellow};
use std::process::Command;

use crate::manager::InstanceManager;
use crate::downloader::Downloader;
use crate::instance::InstanceType;
use crate::cf::CFProject;
use crate::setup;
use crate::invoker::Invoker;
use crate::util;
use crate::auth::User;

static FORGE_PRE13_ID_BLACKLIST : [&str;2] = ["forge-14.23.5.2838",  "forge-1.12.2-14.23.5.2847"]; 


use std::collections::HashMap;
pub async fn download_installer(instance_path: PathBuf, mc_forge_version: String) {
    
    let mut forge_map = HashMap::new();
    let forge_url = format!(
        "https://files.minecraftforge.net/maven/net/minecraftforge/forge/{}/forge-{}-installer.jar",
        mc_forge_version, mc_forge_version
    );

    let forge_fname = format!("forge-{}-installer.jar", mc_forge_version);

    let mut forge_path = instance_path;
    forge_path.push(forge_fname);

    forge_map.insert(forge_path, forge_url);
    Downloader::new(forge_map)
        .process()
        .await
        .expect("Unable to download file");

}

pub async fn download_headless_installer(instance_path: PathBuf) {
    let mut forge_hl_map = HashMap::new();

    let mut forge_hl_path = instance_path;
    forge_hl_path.push("forge-installer-headless-1.0.1.jar");

    let forge_hl_url = "https://github.com/xfl03/ForgeInstallerHeadless/releases/download/1.0.1/forge-installer-headless-1.0.1.jar".to_string();

    forge_hl_map.insert(forge_hl_path, forge_hl_url);
    Downloader::new(forge_hl_map)
        .process()
        .await
        .expect("Unable to download file");

}

pub fn run_forge_installation(
    instance_path: PathBuf, 
    installer_cp: String, 
    post_13 : bool
    ) {

    if post_13 {
        let args = &[
            "-cp",
            installer_cp.as_str(),
            "me.xfl03.HeadlessInstaller",
            "-installClient",
            ".",
        ];
        // invoke the headless installer
        Command::new("java")
            .args(args)
            .current_dir(instance_path)
            .status()
            .expect("Error occured");
    }else{

        println!("\n\n");
        println!("When prompted by forge, {}: {}", Yellow.paint("ENTER THE FOLLOWING"), instance_path.clone().display());
        // run default installer here
        let args = &[
            "-jar",
            installer_cp.as_str(),
        ];

        Command::new("java")
            .args(args)
            .current_dir(instance_path)
            .status()
            .expect("Error occured");
    }

}

pub async fn setup(mut ima: InstanceManager, id: u64, user_path: PathBuf) {
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

    setup::get_modslist(proj.files[choice].clone(), instance.clone()).await;

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

    let mut modloader = manifest_json["minecraft"]["modLoaders"][0]["id"]
        .as_str()
        .unwrap();
    

    // check if modloader is in blacklist
    for id in FORGE_PRE13_ID_BLACKLIST.iter() {
        if *id == modloader {
            modloader =  "forge-14.23.5.2855";
        }
    }


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

    download_installer(instance.get_path(), mc_forge_version.clone()).await;

    // forge headless installer for 1.13.2+
    // no need for headless installer here as forge
    // supports -installClient pre 1.13.2
    let is_pre_13 = !util::geq_version(mcv, "1.13.2");

    if is_pre_13 {
        let installer_cp = match cfg!(windows) {
            true => format!("forge-{}-installer.jar",mc_forge_version),
            false => format!("forge-{}-installer.jar",mc_forge_version)
        };

        run_forge_installation(instance.get_path(), installer_cp, false);
    }else{
        info!("POST 1.13.2");
        download_headless_installer(instance.get_path()).await;
        let installer_cp = match cfg!(windows) {
            true => format!("forge-{}-installer.jar;forge-installer-headless-1.0.1.jar",mc_forge_version),
            false =>  format!("forge-{}-installer.jar:forge-installer-headless-1.0.1.jar",mc_forge_version)
        };

        run_forge_installation(instance.get_path(), installer_cp, true);
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
        setup::get_binaries(vvpc, ip).await;
    });
    
    
    // get libraries for both vanilla and forge
    let vvpc = vanilla_version_path.clone();
    let fvpc = forge_version_path.clone();

    // get a list of all downloads
    let mut downloads = setup::get_library_downloads(libpath.clone(), vvpc).await.unwrap();
    downloads.extend(setup::get_library_downloads(libpath.clone(), fvpc).await.unwrap());

    let mpc = mods_path.clone();
    let mcvc = mcv.to_owned();
    downloads.extend(setup::get_mod_downloads(mcvc, mpc).await.unwrap());
    downloads.extend(
        setup::get_asset_downloads(instance.get_path(), vanilla_version_path)
            .await
            .unwrap()
    );

    let mut downloads_log_file_path = instance.get_path();
    downloads_log_file_path.push("downloads.log");

    // no need for logging anymore

    //let mut downloads_log_file = File::create(downloads_log_file_path).expect("Couldn't create file");

    //for (k,v) in downloads.clone().into_iter() {
    //    let line = format!("{},{}\n", k.display(),v);
    //    downloads_log_file.write(line.as_bytes()).expect("Error writing to file");
    //}

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

    let classes = setup::get_cp_from_version(PathBuf::from("libraries"), version_paths);
    let mut classpaths: Vec<PathBuf> = Vec::new();

    for class in classes {
        classpaths.push(class.1);
    }

    if is_pre_13 {
        // Game args look like this
        //"minecraftArguments": "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --tweakClass net.minecraftforge.fml.common.launcher.FMLTweaker --versionType Forge", 

        let mut args = forge_json["minecraftArguments"].as_str().unwrap().to_string();

        // download text2speech narrator 1.10.2
       
        // THIS IS VERY HACKY
        // but a fix for now...
        let mut download_map = HashMap::new();
        let mut narrator_path = instance.get_path();
        narrator_path.push("libraries/com/mojang/text2speech/1.10.3/text2speech-1.10.3.jar");
        let narrator_url =  "https://libraries.minecraft.net/com/mojang/text2speech/1.10.3/text2speech-1.10.3.jar".to_string();
        download_map.insert(narrator_path, narrator_url);

        Downloader::new(download_map)
            .process()
            .await
            .unwrap();

        // com/mojang/text2speech/1.10.3/text2speech-1.10.3.jar
        //https://libraries.minecraft.net/com/mojang/text2speech/1.10.3/text2speech-1.10.3.jar 
        
        // build game args 
        args = args.replace("${auth_player_name}", user.name.as_str());
        args = args.replace("${version_name}", proj.files[choice].version.as_str());
        args = args.replace("${game_directory}", instance.get_path().to_str().unwrap());
        args = args.replace("${assets_root}", assets_path.to_str().unwrap());
        args = args.replace("${assets_index_name}", asset_index);
        args = args.replace("${auth_uuid}", user.id.as_str());
        args = args.replace("${auth_access_token}", user.token.as_str());
        args = args.replace("${user_type}", "mojang");

        // First time setup
        let mut invoker = Invoker::new(
                "java".to_string(),
                binpath.clone(),
                classpaths.clone(),
                args,
                main_class.to_string(),
                instance.name(),
                InstanceType::Forge,
                user.name,
                user.token
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
                format!("{} --assetsDir {} --assetIndex {} --gameDir {} --version  {}  --versionType release --userType mojang", forge_args.unwrap(), assets_path.display(), asset_index, instance.get_path().display(), proj.files[choice].version),
                main_class.to_string(),
                instance.name(),
                InstanceType::Forge,
                user.name,
                user.token,
            );

        let mut invoker_file_path = instance.get_path();
        invoker_file_path.push("sml_invoker.json");

        invoker.export_as_json(invoker_file_path);

    }

    info!("{}", Green.paint("Setup is complete!"));
}
