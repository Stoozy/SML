use crate::downloader::Downloader;
use std::path::PathBuf;
use std::process::Command;
use ansi_term::Color::Yellow;

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
