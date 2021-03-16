extern crate clap;
extern crate ftp;

use std::fs;
use std::path::PathBuf;

mod cf;
mod downloader;
mod ima;
mod sml;

use clap::*;

use ima::InstanceManager;
use sml::Invoker;

fn get_instances_path() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|mut pb| {
        pb.pop();
        pb.push("instances/");
        // if path dne then, create one
        // (this should be first time only)
        if !pb.exists() {
            println!("Creating instances dir at {}", pb.display());
            fs::create_dir(pb.as_path().clone()).expect("Error creating instances folder");
        }
        Some(pb)
    })
}

fn main() {
    let mut ima = InstanceManager::new(get_instances_path().unwrap());

    // create new app
    let app = App::new("SML")
        .version("1.0")
        .author("Stoozy")
        .about("A Minecraft Modded Launcher CLI")
        .arg(
            Arg::with_name("id")
                .short("i")
                .long("id")
                .help("Searches for project in curseforge with given id")
                .takes_value(true),
        )
        .get_matches();

    match app.value_of("id") {
        Some(id) => {
            let mut proj = cf::CFProject::new(
                id.parse::<u64>().expect("Not a valid id"),
                "https://api.cfwidget.com/".to_string());

            let choice = proj.get_choice();
            let instance = ima
                .create_instance(proj.files[choice].display.clone())
                .expect("Error creating instance");
           
            //sml::get_assets(instance.get_path().clone(), vanilla_version_path.clone()).unwrap();
            sml::get_stage(proj.files[choice].clone(), instance.clone());
            sml::get_modslist(proj.files[choice].clone(), instance.clone());

            let mut libpath = instance.get_path().clone();
            libpath.push("libraries/");

            let mut binpath = instance.get_path().clone();
            binpath.push("bin/");

            let mut assetspath = instance.get_path().clone();
            assetspath.push("assets/");

            let mut version_paths = Vec::new();
            
            // get this programatically later
            let mut forge_version_path = instance.get_path().clone();
            forge_version_path.push("versions/1.16.4-forge-35.1.4/1.16.4-forge-35.1.4.json");

 
            let mut vanilla_version_path = instance.get_path().clone();
            vanilla_version_path.push("versions/1.16.4/1.16.4.json");
            
            version_paths.push(forge_version_path);
            version_paths.push(vanilla_version_path);

            let classpaths = sml::get_cp_from_version(libpath, version_paths);

            let user =  sml::handle_auth().expect("Couldn't get access token");
            let access_token = user.token;


            let mut vanilla_version_path = instance.get_path().clone();
            vanilla_version_path.push("versions/1.16.4/1.16.4.json");

            let mut asset_index = proj.files[choice].version.clone();
            // remove the last . and number
            asset_index.remove(asset_index.len()-1);
            asset_index.remove(asset_index.len()-1);

            let mut invoker = Invoker::new(
                "java -Dminecraft.launcher.brand=minecraft-launcher -Dminecraft.launcher.version=2.2.2012".to_string(),
                binpath,
                classpaths,
                format!("--launchTarget fmlclient  --fml.forgeVersion 35.1.4 --fml.mcVersion 1.16.4 --fml.forgeGroup net.minecraftforge --fml.mcpVersion 20201102.104115 --assetsDir \"{}\" --assetIndex {} --gameDir \"{}\" --version  {} --accessToken {} --versionType release --userType mojang", assetspath.display(), asset_index, instance.get_path().display(), proj.files[choice].version, access_token),
                "cpw.mods.modlauncher.Launcher".to_string(),
                );




            invoker.gen_invocation();
            invoker.display_invocation();



        },
        None => {
            sml::handle_auth();
        }
    };
}
