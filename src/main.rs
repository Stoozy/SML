extern crate clap;
use clap::{App, Arg};
use serde_json::{Map, Value};
use std::io;
//use std::{io::stdout, str, io::Write};

struct CFFile {
    id: u64,
    display: String,
    name: String,
    ftype: String,
    version: String,
}

struct CFProject {
    id: u64,
    api_url: String,
    files: Vec<CFFile>,
}

impl CFProject {
    pub fn new(pid: u64, url: String) -> CFProject {
        CFProject {
            id: pid,
            api_url: url,
            files: Vec::new(),
        }
    }

    pub fn get_json(&mut self) -> serde_json::Value {
        // get proper endpoint
        let body: String = ureq::get(format!("{}{}", self.api_url, self.id).as_str())
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        serde_json::from_str(body.as_str()).unwrap()
    }
}

fn get_u64() -> Option<u64> {
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("Failed to get input");

    Some(
        input_text
            .trim()
            .parse::<u64>()
            .expect("Error parsing number"),
    )
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
            let mut proj = CFProject::new(
                id.parse::<u64>().expect("Not a valid id"),
                "https://api.cfwidget.com/".to_string(),
            );

            let res: Value = proj.get_json();

            let files: &Vec<Value> = res["files"]
                .as_array()
                .expect("Error getting files: Invalid json");

            for (i, fileobj) in files.iter().enumerate() {
                let cfile = CFFile {
                    id: fileobj["id"].as_u64().unwrap(),
                    display: fileobj["display"].as_str().unwrap().to_string(),
                    name: fileobj["name"].as_str().unwrap().to_string(),
                    ftype: fileobj["type"].as_str().unwrap().to_string(),
                    version: fileobj["version"].as_str().unwrap().to_string(),
                };

                proj.files.push(cfile);

                println!(
                    "  [{}]: {} - {}@{}",
                    i,
                    fileobj["display"].as_str().unwrap(),
                    fileobj["type"].as_str().unwrap(),
                    fileobj["version"].as_str().unwrap()
                );
            }

            println!("Choose file (Enter a number): ");

            let choice: u64 = match get_u64() {
                Some(n) => n,
                None => 0,
            };

            let choice = choice as usize;
            println!(
                "You chose {} - {}@{}",
                proj.files[choice].display, proj.files[choice].ftype, proj.files[choice].version
            );
        }
        None => {
            println!("No id was provided.");
        }
    }

    Ok(())
}
