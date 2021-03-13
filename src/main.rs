extern crate clap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

extern crate ftp;

use clap::{App, Arg};
use ftp::FtpStream;

mod cf;
mod downloader;
mod ima;

use downloader::Downloader;
use ima::InstanceManager;
use zip::ZipArchive;

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
            let instance = ima
                .create_instance(proj.files[choice].display.clone())
                .expect("Error creating instance");

            // TODO: implement for windows
            if cfg!(unix) {
                let stage_file_remote_path =
                    format!("/shares/U/sml/{}-linux.zip", proj.files[choice].version);

                let mut stage_filepath = instance.get_path();
                stage_filepath.pop();

                stage_filepath.push(format!("{}-linux.zip", proj.files[choice].version));
                println!("MC Version is: {}", proj.files[choice].version);

                // request server for sml stage
                let mut ftp_stream = FtpStream::connect("98.14.42.52:21").unwrap();
                let _ = ftp_stream.login("", "").unwrap();

                let stage_file_stream = ftp_stream
                    .simple_retr(stage_file_remote_path.as_str())
                    .unwrap();

                match fs::File::create(stage_filepath.clone()) {
                    Err(e) => panic!("Couldn't create file {}", e),
                    Ok(mut file) => {
                        file.write(&stage_file_stream.into_inner()).unwrap();
                    }
                };

                let file = fs::File::open(stage_filepath.clone()).expect("Error getting zip file");
                let mut zip = ZipArchive::new(file).unwrap();
                let extract_path = instance.get_path();
                //extract_path.pop();

                zip.extract(extract_path).expect("Error extracting forge");
                println!("Sucessfully extracted forge stage");
                println!("Cleaning up");
                fs::remove_file(stage_filepath).expect("Error deleting stage zip file");
            }

            let mut download_path = instance.get_path();
            download_path.push("mods/");
            if !download_path.exists() {
                fs::create_dir(download_path.clone()).expect("Error creating mods folder");
            }
            download_path.push(proj.files[choice].name.clone());

            println!("Got download url {}", download_url);
            println!("Got download path {}", download_path.display());

            let mut downloader = Downloader::new(download_url, download_path.clone());
            downloader.download().expect("Error downloading modpack");


            let modpack_zip = fs::File::open(download_path.clone()).expect("Couldn't open modpack");
            println!("Downloaded mods list");

            println!("Extracting mods list");
            let mut zip = ZipArchive::new(modpack_zip).unwrap();
            let mut extract_path = download_path.clone();
            extract_path.pop();


            zip.extract(extract_path).expect("Error extracting mods list");

            fs::remove_file(download_path.clone()).expect("Error deleting stage zip file");

        }
        None => {
            println!("No id was provided.");
        }
    }

    Ok(())
}
