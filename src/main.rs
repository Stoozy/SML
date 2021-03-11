extern crate clap;
use std::path::PathBuf;
use std::fs;

use clap::{App, Arg};
mod cf;
mod downloader;
mod ima;


use downloader::Downloader;
use ima::InstanceManager;

fn get_instances_path() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|mut pb| {
        pb.pop();
        pb.push("instances/");
        if !pb.exists(){
            println!("Creating instances dir at {}", pb.display());
            fs::create_dir(pb.as_path().clone()).expect("Error creating instances folder");
        }
        Some(pb)
    })
}

fn main() -> Result<(), ureq::Error> {
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
                "https://api.cfwidget.com/".to_string(),
            );

            let choice = proj.get_choice();
            let download_url = proj.files[choice].get_download_url();
            let instance = ima.create_instance(proj.files[choice].display.clone());

            let mut download_path = instance.expect("Error creating instance").get_path();
            // add filename to dir
            download_path.push(proj.files[choice].name.clone());

            println!("Got download url {}", download_url);
            println!("Got download path {}", download_path.display());

            let mut downloader = Downloader::new(download_url, download_path);
            downloader.download().expect("Error downloading file");
        }
        None => {
            println!("No id was provided.");
        }
    }

    Ok(())
}
