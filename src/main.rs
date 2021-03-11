extern crate clap;
use std::path::PathBuf;

use clap::{App, Arg};
mod cf;
mod downloader;

use downloader::Downloader;
fn get_exec_name() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|mut pb| {
            pb.pop();
            Some(pb)
        })
}

fn main() -> Result<(), ureq::Error> {
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
            let mut download_path = get_exec_name().unwrap();
            download_path.set_file_name(proj.files[choice].name.clone());

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
